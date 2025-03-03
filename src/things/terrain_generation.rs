// translation of https://github.com/sftd/blender-addons/blob/master/add_mesh_ant_landscape.py
// into rust

use cgmath::{Matrix4, One, Point2, Point3, Vector3};
use genmesh::{MapToVertices, Neighbors, Polygon, Quad, Triangle, Triangulate, Vertices};
use mint::Vector3 as MintVector3;
use rand_distr::{Distribution, UnitSphere};
use profiling;
use crate::render::MyMesh;
use crate::things::hetero_terrain::hetero_terrain_new_perlin;
use crate::utils::{Face, Vertex};

fn landscape_gen(
  x: f32,
  y: f32,
  z: f32,
  rseed: i32,
  nsize: f32,
  depth: f32,
  dimension: f32,
  lacunarity: f32,
  offset: f32,
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
    ncoords.0, ncoords.1, ncoords.2, dimension, /*-H*/
    lacunarity, depth, /* octaves */
    offset,
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

fn create_faces(out_faces: &mut Vec<Face>, vert_idx_1: &[u32], vert_idx_2: &[u32]) {
  /*
  # A very simple "bridge" tool.
  # Connects two equally long vertex rows with faces.
  # Returns a list of the new faces (list of  lists)
  #
  # vert_idx_1 ... First vertex list (list of vertex indices).
  # vert_idx_2 ... Second vertex list (list of vertex indices).
  # closed ... Creates a loop (first & last are closed).
  # flipped ... Invert the normal of the face(s).
  #
  # Note: You can set vert_idx_1 to a single vertex index to create
  #    a fan/star of faces.
  # Note: If both vertex idx list are the same length they have
  #    to have at least 2 vertices.
  def createFaces(vert_idx_1, vert_idx_2, closed=False, flipped=False):
      faces = []

      if not vert_idx_1 or not vert_idx_2:
          return None

      if len(vert_idx_1) < 2 and len(vert_idx_2) < 2:
          return None

      fan = False
      if (len(vert_idx_1) != len(vert_idx_2)):
          if (len(vert_idx_1) == 1 and len(vert_idx_2) > 1):
              fan = True
          else:
              return None

      total = len(vert_idx_2)

      if closed:
          # Bridge the start with the end.
          if flipped:
              face = [
                  vert_idx_1[0],
                  vert_idx_2[0],
                  vert_idx_2[total - 1]]
              if not fan:
                  face.append(vert_idx_1[total - 1])
              faces.append(face)

          else:
              face = [vert_idx_2[0], vert_idx_1[0]]
              if not fan:
                  face.append(vert_idx_1[total - 1])
              face.append(vert_idx_2[total - 1])
              faces.append(face)

      # Bridge the rest of the faces.
      for num in range(total - 1):
          if flipped:
              if fan:
                  face = [vert_idx_2[num], vert_idx_1[0], vert_idx_2[num + 1]]
              else:
                  face = [vert_idx_2[num], vert_idx_1[num],
                      vert_idx_1[num + 1], vert_idx_2[num + 1]]
              faces.append(face)
          else:
              if fan:
                  face = [vert_idx_1[0], vert_idx_2[num], vert_idx_2[num + 1]]
              else:
                  face = [vert_idx_1[num], vert_idx_2[num],
                      vert_idx_2[num + 1], vert_idx_1[num + 1]]
              faces.append(face)

      return faces
  */
  if vert_idx_1.is_empty() || vert_idx_2.is_empty() {
    panic!(
      "lengths should be positive but they are {:?} {:?}",
      vert_idx_1.len(),
      vert_idx_2.len()
    );
  }
  if vert_idx_1.len() < 2 && vert_idx_2.len() < 2 {
    panic!(
      "lengths cant be both less than 2 but they are {:?} {:?}",
      vert_idx_1.len(),
      vert_idx_2.len()
    );
  }
  let mut fan = false;
  if vert_idx_1.len() != vert_idx_2.len() {
    if vert_idx_1.len() == 1 && vert_idx_2.len() > 1 {
      fan = true
    } else {
      panic!(
        "if len first is 1 len  the second should be more than 1 but they are {:?} {:?}",
        vert_idx_1.len(),
        vert_idx_2.len()
      );
    }
  }
  let total = vert_idx_2.len();
  if !fan {
    out_faces.push(Polygon::PolyQuad(Quad::new(
      vert_idx_2[0],
      vert_idx_1[0],
      vert_idx_1[total - 1],
      vert_idx_2[total - 1],
    )));
  } else {
    out_faces.push(Polygon::PolyTri(Triangle::new(
      vert_idx_2[0],
      vert_idx_1[0],
      vert_idx_2[total - 1],
    )));
  }
  for num in 0..(total - 1) {
    if fan {
      out_faces.push(Polygon::PolyTri(Triangle::new(
        vert_idx_1[0],
        vert_idx_2[num],
        vert_idx_2[num + 1],
      )));
    } else {
      out_faces.push(Polygon::PolyQuad(Quad::new(
        vert_idx_1[num],
        vert_idx_2[num],
        vert_idx_2[num + 1],
        vert_idx_1[num + 1],
      )));
    }
  }
}

fn grid_gen(
  sub_division: i32,
  mesh_size: i32,
  oleft: Option<Vec<f32>>,
  oright: Option<Vec<f32>>,
  otop: Option<Vec<f32>>,
  obottom: Option<Vec<f32>>,
) -> (
  Vec<Vertex>,
  Vec<Face>,
  Vec<f32>,
  Vec<f32>,
  Vec<f32>,
  Vec<f32>,
) {
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
  let mut left: Vec<f32> = vec![];
  let mut right: Vec<f32> = vec![];
  let mut top: Vec<f32> = vec![];
  let mut bottom: Vec<f32> = vec![];
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
      let depth = 8.0;
      let dimension = 0.95;
      let lacunarity = 2.20;
      let offset = 0.50;
      let height = 0.23;
      let heightoffset = 0.0;
      let sealevel = -1.0;
      let platlevel = 1.0;
      let mut z = landscape_gen(
        x,
        y,
        0.0,
        1,
        nsize,
        depth,
        dimension,
        lacunarity,
        offset,
        height,
        heightoffset,
        sealevel,
        platlevel,
      );
      if row_x == 0 {
        if let Some(o) = oleft.as_ref().unwrap_or(&vec![]).get(row_y as usize) {
          z = *o;
        }
        left.push(z);
      }
      if row_x == sub_division - 1 {
        if let Some(o) = oright.as_ref().unwrap_or(&vec![]).get(row_y as usize) {
          z = *o;
        }
        right.push(z);
      }
      if row_y == 0 {
        if let Some(o) = otop.as_ref().unwrap_or(&vec![]).get(row_x as usize) {
          z = *o;
        }
        top.push(z);
      }
      if row_y == sub_division - 1 {
        if let Some(o) = obottom.as_ref().unwrap_or(&vec![]).get(row_x as usize) {
          z = *o;
        }
        bottom.push(z);
      }
      edgeloop_cur.push(verts.len() as u32);
      verts.push(Vertex {
        position: (x, y, z),
        tex: (-1.0, -1.0),
        tex_offset: (0, 0),
      });
    }
    if !edgeloop_prev.is_empty() {
      create_faces(&mut faces, &edgeloop_prev, &edgeloop_cur);
    }
    edgeloop_prev = edgeloop_cur;
  }
  (verts, faces, left, right, top, bottom)
}

#[derive(Clone)]
pub struct TerrainModel {
  pub mesh: MyMesh,
  pub left: Vec<f32>,
  pub right: Vec<f32>,
  pub top: Vec<f32>,
  pub bottom: Vec<f32>,
}

#[profiling::function]
pub fn terrain_execute(
  scale: f32,
  sub_division: i32,
  mesh_size: i32,
  x: f32,
  z: f32,
  oleft: Option<Vec<f32>>,
  oright: Option<Vec<f32>>,
  otop: Option<Vec<f32>>,
  obottom: Option<Vec<f32>>,
) -> TerrainModel {
  let (verts, faces, left, right, top, bottom) =
    grid_gen(sub_division, mesh_size, oleft, oright, otop, obottom);
  let vertex: Vec<Point3<f32>> = verts
    .iter()
    .map(|v| Point3::new(v.position.0, v.position.2 * scale, v.position.1))
    .collect();

  let triangles: Vec<Triangle<usize>> = faces
    .iter()
    .cloned()
    .triangulate()
    .vertex(|v| v as usize)
    .collect();

  let neighbours = Neighbors::new(vertex.clone(), triangles.clone());

  let normals: Vec<Point3<f32>> = (0..vertex.len())
    .map(|i| neighbours.normal_for_vertex(i, |v| MintVector3::<f32>::from([v.x, v.y, v.z])))
    .map(|v| Point3::from((-v.x, -v.y, -v.z)))
    .collect();

  let index: Vec<u32> = triangles
    .iter()
    .cloned()
    .vertices()
    .map(|v| v as u32)
    .collect();

  let tex = (0..vertex.len())
    .map(|_i| Point2::new(-1.0, -1.0))
    .collect();
  let tex_offset = (0..vertex.len()).map(|_i| Point2::new(0, 0)).collect();
  let transform = <Matrix4<f32> as One>::one();

  let mut mesh = MyMesh::new(vertex, tex, tex_offset, normals, index, transform, false);
  mesh.update_transform_2(
    Vector3::new(x, 0.0, z),
    Matrix4::<f32>::one(),
    //Matrix4::from_angle_x(Rad(std::f32::consts::FRAC_PI_2)),
    [1.0, 1.0, 1.0],
  );
  TerrainModel {
    mesh: mesh,
    left,
    right,
    top,
    bottom,
  }
}
