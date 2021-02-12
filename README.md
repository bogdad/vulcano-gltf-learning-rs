# vulkano-gltf-learning-rs
[https://github.com/bogdad/vulkano-gltf-learning-rs](https://github.com/bogdad/vulkano-gltf-learning-rs)

## Questions:

- [ ] how do we write text on things in 3d
- [x] how do we generate random "cloud like landcapes" on the fly
- [ ] how do vertex normals work
- [x] how do coordinate systems work

## how do we generate random "cloud like landcapes" on the fly

![screenshot](./images/4.png)

So blender uses some kind of random perlin noise mesh generator, see [add mesh ant landscape](https://github.com/sftd/blender-addons/blob/master/add_mesh_ant_landscape.py) which in turn uses some c written things, see [blendlib intern noise](https://github.com/blender/blender/blob/594f47ecd2d5367ca936cf6fc6ec8168c2b360d0/source/blender/blenlib/intern/noise.c#L1462)
which i had to translate to rust.

then i had to learn how to dynamically supply newly generated landcape meshes from the background thread to the main game loop. pretty neat, but still parts left to do. like, dynamically determine the range for pre-generating terrain based on camera fov area? i.e. generate meshes from here and till the end of visible region. And would be cool to make the generated regions of terrain to be "seamless".

## how do coordinate systems work:
Gltf default camera: “and the default camera sits on the -Z side looking toward the origin with +Y up”

![screenshot](./images/2.png)

(Dimensions of the box (x:1 y:4 z:16)
But:

![screenshot](./images/3.png)

I.e. blender gltf export swaps y and z. Weird.

What is blender convention:
* The /X-axis/ typically represents side-to-side movement.
* The /Y-axis/ represents front-to-back movement.
* The /Z-axis/ goes from top to bottom.

So, `blender gltf exporter` converts coordinate systems: swaps Y and Z. 
If you are designing an asset in blender to be viewable in the default camera in gltf (sitting in -Z:`[0, 0, -1]` pointing towards +Z `[0, 0, 1]` pointing up and with +Y`[0, 1, 0]`
