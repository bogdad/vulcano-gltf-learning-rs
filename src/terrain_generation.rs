// translation of https://github.com/sftd/blender-addons/blob/master/add_mesh_ant_landscape.py
// into rust

use genmesh::{Polygon, Quad, Triangle, Triangulate, Vertices};

use cgmath::prelude::*;
use cgmath::{Matrix4, Point3};
use rand;
use rand_distr::{Distribution, UnitSphere};

use crate::hetero_terrain::hetero_terrain_new_perlin;
use crate::render::MyMesh;
use crate::utils::{Face, Vertex};

fn landscape_gen(
    x: f32,
    y: f32,
    z: f32,
    mesh_size: i32,
    rseed: i32,
    nsize: f32,
    nbasis: f32,
    depth: f32,
    dimension: f32,
    lacunarity: f32,
    offset: f32,
    invert: f32,
    height: f32,
    heightoffset: f32,
    sealevel: f32,
    platlevel: f32,
) -> f32 {
    /*
    options=[0,1.0,1, 0,0,1.0,0,6,1.0,2.0,1.0,2.0,0,0,0, 1.0,0.0,1,0.0,1.0,0,0,0]
        # options
        rseed    = options[0]
        nsize    = options[1]
        ntype      = int( options[2][0] )
        nbasis     = int( options[3][0] )
        vlbasis    = int( options[4][0] )
        distortion = options[5]
        hardnoise  = options[6]
        depth      = options[7]
        dimension  = options[8]
        lacunarity = options[9]
        offset     = options[10]
        gain       = options[11]
        marblebias     = int( options[12][0] )
        marblesharpnes = int( options[13][0] )
        marbleshape    = int( options[14][0] )
        invert       = options[15]
        height       = options[16]
        heightoffset = options[17]
        falloff      = int( options[18][0] )
        sealevel     = options[19]
        platlevel    = options[20]
        strata       = options[21]
        stratatype   = options[22]
        sphere       = options[23]

        # origin
        if rseed == 0:
            origin = 0.0,0.0,0.0
            origin_x = 0.0
            origin_y = 0.0
            origin_z = 0.0
        else:
            # randomise origin
            seed_set( rseed )
            origin = random_unit_vector()
            origin_x = ( 0.5 - origin[0] ) * 1000.0
            origin_y = ( 0.5 - origin[1] ) * 1000.0
            origin_z = ( 0.5 - origin[2] ) * 1000.0

        # adjust noise size and origin
        ncoords = ( x / nsize + origin_x, y / nsize + origin_y, z / nsize + origin_z )

        # noise basis type's
        if nbasis == 9: nbasis = 14  # to get cellnoise basis you must set 14 instead of 9
        if vlbasis ==9: vlbasis = 14
        # noise type's
        if ntype == 0:   value = multi_fractal(        ncoords, dimension, lacunarity, depth, nbasis ) * 0.5
        elif ntype == 1: value = ridged_multi_fractal( ncoords, dimension, lacunarity, depth, offset, gain, nbasis ) * 0.5
        elif ntype == 2: value = hybrid_multi_fractal( ncoords, dimension, lacunarity, depth, offset, gain, nbasis ) * 0.5
        elif ntype == 3: value = hetero_terrain(       ncoords, dimension, lacunarity, depth, offset, nbasis ) * 0.25
        elif ntype == 4: value = fractal(              ncoords, dimension, lacunarity, depth, nbasis )
        elif ntype == 5: value = turbulence_vector(    ncoords, depth, hardnoise, nbasis )[0]
        elif ntype == 6: value = variable_lacunarity(            ncoords, distortion, nbasis, vlbasis ) + 0.5
        elif ntype == 7: value = marble_noise( x*2.0/falloffsize,y*2.0/falloffsize,z*2/falloffsize, origin, nsize, marbleshape, marblebias, marblesharpnes, distortion, depth, hardnoise, nbasis )
        elif ntype == 8: value = shattered_hterrain( ncoords[0], ncoords[1], ncoords[2], dimension, lacunarity, depth, offset, distortion, nbasis )
        elif ntype == 9: value = strata_hterrain( ncoords[0], ncoords[1], ncoords[2], dimension, lacunarity, depth, offset, distortion, nbasis )
        else:
            value = 0.0

        # adjust height
        if invert !=0:
            value = (1-value) * height + heightoffset
        else:
            value = value * height + heightoffset

        # edge falloff
        if sphere == 0: # no edge falloff if spherical
            if falloff != 0:
                fallofftypes = [0, hypot(x * x, y * y), hypot(x, y), abs(y), abs(x)]
                dist = fallofftypes[ falloff]
                if falloff ==1:
                    radius = (falloffsize/2)**2
                else:
                    radius = falloffsize/2
                value = value - sealevel
                if( dist < radius ):
                    dist = dist / radius
                    dist = ( (dist) * (dist) * ( 3-2*(dist) ) )
                    value = ( value - value * dist ) + sealevel
                else:
                    value = sealevel

        # strata / terrace / layered
        if stratatype !='0':
            strata = strata / height
        if stratatype == '1':
            strata *= 2
            steps = ( sin( value*strata*pi ) * ( 0.1/strata*pi ) )
            value = ( value * (1.0-0.5) + steps*0.5 ) * 2.0
        elif stratatype == '2':
            steps = -abs( sin( value*(strata)*pi ) * ( 0.1/(strata)*pi ) )
            value =( value * (1.0-0.5) + steps*0.5 ) * 2.0
        elif stratatype == '3':
            steps = abs( sin( value*(strata)*pi ) * ( 0.1/(strata)*pi ) )
            value =( value * (1.0-0.5) + steps*0.5 ) * 2.0
        else:
            value = value

        # clamp height
        if ( value < sealevel ): value = sealevel
        if ( value > platlevel ): value = platlevel

        return value
    */
    let (origin_x, origin_y, origin_z) = if rseed == 0 {
        (0.0, 0.0, 0.0)
    } else {
        let v: [f32; 3] = UnitSphere.sample(&mut rand::thread_rng());
        (
            (0.5 - v[0]) * 1000.0,
            (0.5 - v[1]) * 1000.0,
            (0.5 - v[2]) * 1000.0,
        )
    };

    let ncoords = (
        x / nsize + origin_x,
        y / nsize + origin_y,
        z / nsize + origin_z,
    );
    let mut value = hetero_terrain_new_perlin(
        ncoords.0,
        ncoords.1,
        ncoords.2,
        dimension, /*-H*/
        lacunarity,
        depth, /* octaves */
        offset,
        nbasis as i32,
    ) * 0.25;

    value = value * height + heightoffset;
    if value < sealevel {
        value = sealevel
    }
    if value > platlevel {
        value = platlevel
    }
    value
}

fn create_faces(out_faces: &mut Vec<Face>, vertIdx1: &Vec<u32>, vertIdx2: &Vec<u32>) {
    /*
    # A very simple "bridge" tool.
    # Connects two equally long vertex rows with faces.
    # Returns a list of the new faces (list of  lists)
    #
    # vertIdx1 ... First vertex list (list of vertex indices).
    # vertIdx2 ... Second vertex list (list of vertex indices).
    # closed ... Creates a loop (first & last are closed).
    # flipped ... Invert the normal of the face(s).
    #
    # Note: You can set vertIdx1 to a single vertex index to create
    #    a fan/star of faces.
    # Note: If both vertex idx list are the same length they have
    #    to have at least 2 vertices.
    def createFaces(vertIdx1, vertIdx2, closed=False, flipped=False):
        faces = []

        if not vertIdx1 or not vertIdx2:
            return None

        if len(vertIdx1) < 2 and len(vertIdx2) < 2:
            return None

        fan = False
        if (len(vertIdx1) != len(vertIdx2)):
            if (len(vertIdx1) == 1 and len(vertIdx2) > 1):
                fan = True
            else:
                return None

        total = len(vertIdx2)

        if closed:
            # Bridge the start with the end.
            if flipped:
                face = [
                    vertIdx1[0],
                    vertIdx2[0],
                    vertIdx2[total - 1]]
                if not fan:
                    face.append(vertIdx1[total - 1])
                faces.append(face)

            else:
                face = [vertIdx2[0], vertIdx1[0]]
                if not fan:
                    face.append(vertIdx1[total - 1])
                face.append(vertIdx2[total - 1])
                faces.append(face)

        # Bridge the rest of the faces.
        for num in range(total - 1):
            if flipped:
                if fan:
                    face = [vertIdx2[num], vertIdx1[0], vertIdx2[num + 1]]
                else:
                    face = [vertIdx2[num], vertIdx1[num],
                        vertIdx1[num + 1], vertIdx2[num + 1]]
                faces.append(face)
            else:
                if fan:
                    face = [vertIdx1[0], vertIdx2[num], vertIdx2[num + 1]]
                else:
                    face = [vertIdx1[num], vertIdx2[num],
                        vertIdx2[num + 1], vertIdx1[num + 1]]
                faces.append(face)

        return faces
    */
    if vertIdx1.len() == 0 || vertIdx2.len() == 0 {
        panic!(
            "lengths should be positive but they are {:?} {:?}",
            vertIdx1.len(),
            vertIdx2.len()
        );
    }
    if vertIdx1.len() < 2 && vertIdx2.len() < 2 {
        panic!(
            "lengths cant be both less than 2 but they are {:?} {:?}",
            vertIdx1.len(),
            vertIdx2.len()
        );
    }
    let mut fan = false;
    if vertIdx1.len() != vertIdx2.len() {
        if vertIdx1.len() == 1 && vertIdx2.len() > 1 {
            fan = true
        } else {
            panic!(
                "if len first is 1 len  the second should be more than 1 but they are {:?} {:?}",
                vertIdx1.len(),
                vertIdx2.len()
            );
        }
    }
    let total = vertIdx2.len();
    if !fan {
        out_faces.push(Polygon::PolyQuad(Quad::new(
            vertIdx2[0],
            vertIdx1[0],
            vertIdx1[total - 1],
            vertIdx2[total - 1],
        )));
    } else {
        out_faces.push(Polygon::PolyTri(Triangle::new(
            vertIdx2[0],
            vertIdx1[0],
            vertIdx2[total - 1],
        )));
    }
    for num in 0..(total - 1) {
        if fan {
            out_faces.push(Polygon::PolyTri(Triangle::new(
                vertIdx1[0],
                vertIdx2[num],
                vertIdx2[num + 1],
            )));
        } else {
            out_faces.push(Polygon::PolyQuad(Quad::new(
                vertIdx1[num],
                vertIdx2[num],
                vertIdx2[num + 1],
                vertIdx1[num + 1],
            )));
        }
    }
}

fn grid_gen(sub_division: i32, mesh_size: i32) -> (Vec<Vertex>, Vec<Face>) {
    /*
    verts = []
        faces = []
        edgeloop_prev = []

        delta = size_me / float(sub_d - 1)
        start = -(size_me / 2.0)

        for row_x in range(sub_d):
            edgeloop_cur = []
            x = start + row_x * delta
            for row_y in range(sub_d):
                y = start + row_y * delta
                z = landscape_gen(x,y,0.0,size_me,options)

                edgeloop_cur.append(len(verts))
                verts.append((x,y,z))

            if len(edgeloop_prev) > 0:
                faces_row = createFaces(edgeloop_prev, edgeloop_cur)
                faces.extend(faces_row)

            edgeloop_prev = edgeloop_cur

        return verts, faces
    */
    let mut verts: Vec<Vertex> = vec![];
    let mut faces: Vec<Face> = vec![];
    let delta = (mesh_size as f32) / ((sub_division - 1) as f32);
    let start = -(mesh_size / 2);
    let mut edgeloop_prev: Vec<u32> = vec![];
    for row_x in 0..sub_division {
        let mut edgeloop_cur: Vec<u32> = vec![];
        let x = (start as f32) + (row_x as f32) * delta;
        for row_y in 0..sub_division {
            let y = (start as f32) + (row_y as f32) * delta;
            /*

                fn landscape_gen(x: f32, y:f32, z:f32,mesh_size: i32,
            rseed: i32, nsize: f32, nbasis: f32, depth: f32, dimension: f32, lacunarity: f32,
            offset: f32, invert: f32, height:f32, heightoffset:f32, sealevel: f32, platlevel: f32) -> f32 {
            */
            let nsize = 0.33;
            let nbasis = 0.0;
            let depth = 8.0;
            let dimension = 0.95;
            let lacunarity = 2.20;
            let offset = 0.50;
            let invert = 0.0;
            let height = 0.23;
            let heightoffset = 0.0;
            let sealevel = -1.0;
            let platlevel = 1.0;
            let z = landscape_gen(
                x,
                y,
                0.0,
                mesh_size,
                0,
                nsize,
                nbasis,
                depth,
                dimension,
                lacunarity,
                offset,
                invert,
                height,
                heightoffset,
                sealevel,
                platlevel,
            );
            edgeloop_cur.push(verts.len() as u32);
            verts.push(Vertex {
                position: (x, y, z),
            });
        }
        if edgeloop_prev.len() > 0 {
            create_faces(&mut faces, &edgeloop_prev, &edgeloop_cur);
        }
        edgeloop_prev = edgeloop_cur;
    }
    (verts, faces)
}

pub fn execute(sub_division: i32, mesh_size: i32) -> MyMesh {
    let (verts, faces) = grid_gen(sub_division, mesh_size);
    let vertex: Vec<Point3<f32>> = verts
        .iter()
        .map(|v| Point3::new(v.position.0, v.position.1, v.position.2))
        .collect();
    let normals: Vec<Point3<f32>> = verts
        .iter()
        .map(|v| Point3::new(v.position.0, v.position.1, v.position.2))
        .collect();
    let iter: std::slice::Iter<genmesh::Polygon<u32>> = faces.iter();
    let iter_cloned: std::iter::Cloned<std::slice::Iter<genmesh::Polygon<u32>>> = iter.cloned();
    let index: Vec<u32> = iter_cloned.triangulate().vertices().collect();
    let transform = <Matrix4<f32> as One>::one();
    MyMesh {
        vertex,
        normals,
        index,
        transform,
    }
}
