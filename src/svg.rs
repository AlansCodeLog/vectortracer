use visioncortex::{Color, CompoundPath, PointF64};

pub struct Svg {
	pub paths: Vec<String>,
	pub options: SvgOptions,
}
pub struct SvgOptions {
	pub scale: f32,
	pub backgroundColor: Option<String>,
	pub pathFill: Option<String>,
	pub attributes: Option<String>,
}
/**
Constructs a "dumb" string only svg.
Real elements aren't used so that this can run in a webworker.
*/
impl Svg {
	pub fn new(options: SvgOptions) -> Self {
		let paths = vec![];
		Self { paths, options }
	}

	pub fn add_path(&mut self, paths: &CompoundPath, color: &Color, precision: Option<u32>) {
		let (string, offset) = paths.to_svg_string(true, PointF64::default(), precision);
		// log(format!("{:#?}", paths).as_str());
		// log(format!("{:#?}", paths.paths).as_str());
		let defaultFill = &color.to_hex_string();
		let fillColor = self.options.pathFill.as_ref().unwrap_or(defaultFill);
		let path = format!(
			r#"
                <path
                    d="{d}"
                    transform="translate({x},{y})"
                    style="fill:{fill}"
                />
            "#,
			d = &string,
			x = offset.x,
			y = offset.y,
			fill = fillColor
		);
		self.paths.push(path)
	}
	pub fn get_svg_string(&self) -> String {
		let defaultBg = &"white".to_string();
		let bg = self.options.backgroundColor.as_ref().unwrap_or(defaultBg);
		let defaultAttributes = &"".to_string();
		let attributes = self
			.options
			.attributes
			.as_ref()
			.unwrap_or(defaultAttributes);
		let extra_space = if (attributes.len() > 0) { "" } else { " " };
		let res = format!(
			r#"
                <svg xmlns="http://www.w3.org/2000/svg" style="background:{background};"{extra_space}{attributes}>
                    <g transform="scale({scale})">
                        {paths}
                    </g>
                </svg>
            "#,
			background = bg,
			paths = self.paths.join(""),
			scale = self.options.scale,
			attributes = attributes,
			extra_space = extra_space
		);
		return res;
	}
}
