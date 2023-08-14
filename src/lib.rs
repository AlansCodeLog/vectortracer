#![allow(non_snake_case)]
#![allow(unused_parens)]
use web_sys::{console, ImageData};
mod svg;
mod utils;
use serde::{Deserialize, Serialize};
use svg::*;
use tsify::Tsify;
use visioncortex::{
	clusters::Clusters, BinaryImage, Color, ColorImage, ColorName, PathSimplifyMode,
};
use wasm_bindgen::prelude::*;

#[allow(dead_code)]
fn log(string: &str) {
	console::log_1(&wasm_bindgen::JsValue::from_str(string));
}
#[wasm_bindgen(start)]
pub fn main() {
	utils::set_panic_hook();
	console_log::init().unwrap();
}
pub fn path_simplify_mode(s: &str) -> PathSimplifyMode {
	match s {
		"polygon" => PathSimplifyMode::Polygon,
		"spline" => PathSimplifyMode::Spline,
		"none" => PathSimplifyMode::None,
		_ => panic!("unknown PathSimplifyMode {}", s),
	}
}

#[derive(Tsify, Debug, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct RawImageData {
	#[tsify(type = "Uint8ClampedArray")]
	pub data: Vec<u8>,
	pub width: usize,
	pub height: usize,
}

#[derive(Debug)]
pub struct DebugImageData {
	pub data_len: usize,
	pub first_val: bool,
	pub width: usize,
	pub height: usize,
}

// these are the defults used in vtracer's demo app

fn default_mode() -> String { "spline".to_string() }
fn default_scale() -> f32 { 1.0 }
fn default_cornerThreshold() -> f64 { 60.0_f64.to_radians() }
fn default_lengthThreshold() -> f64 { 4.0 }
fn default_maxIterations() -> usize { 10 }
fn default_spliceThreshold() -> f64 { 45.0_f64.to_radians() }
fn default_filterSpeckle() -> usize { 4 }
fn default_pathPrecision() -> u32 { 8 }


#[derive(Tsify, Debug, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct BinaryImageConverterParams {
	pub debug: Option<bool>,
	/** Default is spline.
     * none = pixel
     */
	#[tsify(type = "'polygon'|'spline'|'none'")]
	#[serde(default = "default_mode")]
	pub mode: String,
    /** Must be in radians. Default is 60deg */
    #[serde(default = "default_cornerThreshold")]
	pub cornerThreshold: f64,
    /** Default is 4. */
    #[serde(default = "default_lengthThreshold")]
	pub lengthThreshold: f64,
    /** Default is 10. */
    #[serde(default = "default_maxIterations")]
	pub maxIterations: usize,
    /** Must be in radians. Default is 45deg */
    #[serde(default = "default_spliceThreshold")]
	pub spliceThreshold: f64,
    /** Default is 4. */
    #[serde(default = "default_filterSpeckle")]
	pub filterSpeckle: usize,
    /** Default is 8. */
    #[serde(default = "default_pathPrecision")]
	pub pathPrecision: u32,
}

#[derive(Tsify, Debug, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Options {
	/** Process an inverted version of the image. */
	pub invert: Option<bool>,
	/** The color to set for the path fill property. By the default this is the color returned by visioncortex's binary converter (i.e. black).*/
	pub pathFill: Option<String>,
	/** The color given to the svg element background, white by default. This is set in a style tag.*/
	pub backgroundColor: Option<String>,
	/** Additional attributes to add to the svg. For now this is a string to simplify things, therefore you cannot specify a style tag, or if you do, you're overriding the default one which contains the background color.*/
	pub attributes: Option<String>,
	/** Create a group and scale the final svg by this amount.*/
	#[serde(default = "default_scale")]
	pub scale: f32,
}

#[wasm_bindgen]
pub struct BinaryImageConverter {
	debug: bool,
	clusters: Clusters,
	counter: usize,
	mode: PathSimplifyMode,
	converterOptions: BinaryImageConverterParams,
	image: BinaryImage,
	svg: Svg,
}

#[wasm_bindgen]
impl BinaryImageConverter {
	#[wasm_bindgen(constructor)]
	// Tsify automatically converts params using serde_wasm_bindgen::from_value(params) where params was JsValue
	pub fn new(
		imageData: ImageData,
		converterOptions: BinaryImageConverterParams,
		options: Options,
	) -> Self {
		let data = imageData.data();
		let len = data.len();
		let colorImage = ColorImage {
			width: imageData.width() as usize,
			height: imageData.height() as usize,
			pixels: data.to_vec(),
		};
        let invert = options.invert.unwrap_or_default();
		let image =
			colorImage.to_binary_image(|x| if invert { x.r > 128 } else { x.r < 128 });
		let debug = converterOptions.debug.is_some_and(|x| x == true);
		if (debug) {
			log(format!("{:#?}", converterOptions).as_str());
			log(format!(
				"{:#?}",
				DebugImageData {
					width: image.width,
					first_val: image.get_pixel_safe(0, 0),
					height: image.height,
					data_len: len 
				}
			)
			.as_str());
		}
		Self {
			debug,
			clusters: Clusters::default(),
			counter: 0,
			mode: path_simplify_mode(&converterOptions.mode),
			image,
			converterOptions,
			svg: Svg::new(SvgOptions {
				scale: options.scale,
				backgroundColor: options.backgroundColor.clone(),
				pathFill: options.pathFill.clone(),
				attributes: options.attributes.clone(),
			}),
		}
	}

	pub fn init(&mut self) {
		self.clusters = self.image.to_clusters(false);
		if (self.debug) {
			log(format!("clusters length {:?}", self.clusters.len()).as_str());
		}
	}

	pub fn tick(&mut self) -> bool {
		if self.counter < self.clusters.len() {
			let cluster = self.clusters.get_cluster(self.counter);
			if cluster.size() >= self.converterOptions.filterSpeckle {
				let paths = cluster.to_compound_path(
					self.mode,
					self.converterOptions.cornerThreshold,
					self.converterOptions.lengthThreshold,
					self.converterOptions.maxIterations,
					self.converterOptions.spliceThreshold,
				);
				let color = Color::color(&ColorName::Black);
				self.svg
					.add_path(&paths, &color, Some(self.converterOptions.pathPrecision));
			} else {
				if (self.debug) {
					log(format!(
						"cluster of size ({:#?}) smaller than filterSpeckle, cluster discarded",
						cluster.size()
					)
					.as_str());
				}
			}
			self.counter += 1;
			false
		} else {
			true
		}
	}
	pub fn getResult(&self) -> String {
		let result = self.svg.get_svg_string();

		if (self.debug) {
			log(&result.as_str());
		};
		result
	}

	pub fn progress(&self) -> u32 {
		100 * self.counter as u32 / self.clusters.len() as u32
	}
}
