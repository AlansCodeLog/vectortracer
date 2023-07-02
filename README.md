![Build](https://github.com/alanscodelog/vectortracer/workflows/Build/badge.svg)
[![Release](https://github.com/alanscodelog/vectortracer/workflows/Release/badge.svg)](https://www.npmjs.com/package/vectortracer)

Wasm bindings to [Visioncortex's VTracer library](https://github.com/visioncortex/vtracer).

# Features

- [x] Compatible with webworkers.
	- [ ] Webworker wrapper example.
- [x] Optional debug mode (note this will be slower).
- [x] Binary Image Converter
	- [x] Option to invert image data.
	- [x] Control over svg path and background color.
	- [x] Arbitrary attributes.
	- [x] Option to scale paths group.
- [ ] Publish
- [ ] Color Image Converter

# Installation

```
npm install vectortracer
```

The package can be installed directly from the git repo like so:

```
npm install https://github.com/alanscodelog/vectortracer
```

# Usage

This is a simple example using a promise with setTimeout.

```ts
import { BinaryImageConverter, BinaryImageConverterParams, Options } from "vectortracer"

export function imageDataToSvg(
	imageData:ImageData,
	vtracerOptions:BinaryImageConverterParams,
	additionalOptions:Options
): string {
	const converter = new BinaryImageConverter(imageData, vtraceroptions, additionalOptions)

	return new Promise(resolve => {
		let done = false
		let progress = 0
		const tick = () => {
			done = converter.tick()
			progress = converter.progress()
			if (!done) {
				setTimeout(tick, 0)
			} else {
				const result = converter.getResult()
				converter.free()
				resolve(result)
			}
		}
		converter.init()
		setTimeout(tick, 0)
	}).catch(err => {
		...
	}) as Promise<string>
}
```

You can then use the function like so:
```ts
const result = await imageDataToSvg(imageData, vtracerOptions, additionalOptions)
// if you have access to the DOM you can parse it to an svg element like so

const parser = new DOMParser()
const svg = parser.parseFromString(res as string, "image/svg+xml").children[0] as SVGElement
```

# Notes

The code is similar, but simpler than the one use in visioncortex's [demo web app code](https://github.com/visioncortex/vtracer/tree/master/webapp). ImageData can be passed directly from js, instead of having to give it a canvas id and having that fragile connection. This means we can use the library from a webworker. What we do lose is the ability to get an SVG element back, again because we don't have access to the DOM in a webworker.

I have not benchmarked it properly yet, but in the app I wrote this for, the transformation seems to take on average a little bit more time (~20ms), but I get more varied times, sometimes faster, sometimes slower, and there are other things running in the background. Still, I think this is the simplest option\* if we want to allow the UI to stay responsive. You can see in the [demo app](https://www.visioncortex.org/vtracer/) that with large images and expensive parameters the UI can freeze despite its use of setTimeout to try and mitigate this. 

To try to avoid any slowdowns from crossing the js/wasm boundary, [Tsify](https://github.com/madonoharu/tsify) with [serde-wasm-bindgen](https://github.com/cloudflare/serde-wasm-bindgen) is used to provide faster serialization/deserialization (it converts js types directly to/from rust) compared to json serialization/deserialization, which is slower.

\* The other possible option is having a wrapper function that takes care of creating a canvas, or using an offscreen canvas in the worker. I wanted to avoid this since for my use case I might not always have a canvas, but I do always have the ImageData.

## Building

There is a root level package json that has the needed prepare script. The `pkg/package.json` is not used. 
