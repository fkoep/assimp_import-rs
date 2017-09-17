extern crate assimp_import as ai;

use std::env;

fn main() {
    let file = match env::args().skip(1).next() {
        Some(r) => r,
        None => {
            println!("Usage: [program] [file]");
            return;
        }
    };

    println!("=== Loading file '{}' ===", &file);
    let scene = ai::Scene::from_file(&file, ai::PostProcessSteps::empty()).unwrap();

    println!("\n=== Loading successful ===");
    println!("Scene flags: {:?}", scene.flags());

    println!("\n=== Nodes ===");
    fn print_node(node: &ai::Node, depth: usize, idx: usize) {
        let indent: String = (0..depth).map(|_| ' ').collect();
        println!("{}| Node #{}", indent, idx);
        println!("{}- Name:\t\t{:?}", indent, node.name());
        println!("{}- Meshes:\t\t{:?}", indent, node.meshes());

        for (idx, child) in node.children().iter().enumerate() {
            print_node(child, depth + 1, idx);
        }
    }
    print_node(&scene.root_node(), 0, 0);

    println!("\n=== Meshes ===");
    for (idx, mesh) in scene.meshes().iter().enumerate() {
        println!("| Mesh #{}", idx);
        println!("- Name:\t\t\t{:?}", mesh.name());
        println!("- Primitive Types:\t{:?}", mesh.primitive_types());
        println!("- Vertices:\t\t{}", mesh.vertices().len());
        println!("- Normals:\t\t{}", mesh.normals().len());
        println!("- Tangents:\t\t{}", mesh.tangents().len());
        println!("- Bitangents:\t\t{}", mesh.bitangents().len());

        for idx in 0..ai::MAX_COLOR_SETS {
            let count = mesh.colors(idx).len();
            if count == 0 {
                continue;
            }
            println!("- Colors[{}]:\t\t{}", idx, count);
        }

        for idx in 0..ai::MAX_TEXTURE_COORDS {
            let count = mesh.texture_coords(idx).len();
            if count == 0 {
                continue;
            }
            println!("- Texture Coords[{}]:\t{}", idx, count);
        }

        for idx in 0..ai::MAX_TEXTURE_COORDS {
            let count = mesh.num_uv_components(idx);
            if count == 0 {
                continue;
            }
            println!("- Uv Components[{}]:\t{}", idx, count);
        }

        println!("- Faces:\t\t{}", mesh.faces().len());
        println!("- Bones:\t\t{}", mesh.bones().len());
        println!("- Material Idx:\t\t{}", mesh.material_idx());
    }

    println!("\n=== Materials ===");
    for (idx, mat) in scene.materials().iter().enumerate() {
        println!("| Material #{}", idx);
        // println!("- Name:\t\t\t{:?}", mat.name());
        // println!("- Properties: {:#?}", mat.properties());

        let props = mat.material_properties();
        println!("-- name: {}", props.name);
        println!("-- twosided: {}", props.twosided);
        println!("-- shading_mode: {:?}", props.shading_mode);
        println!("-- wireframe: {}", props.wireframe);
        println!("-- blend_mode: {:?}", props.blend_mode);
        println!("-- opacity: {}", props.opacity);
        println!("-- bumpscaling: {}", props.bumpscaling);
        println!("-- shininess: {}", props.shininess);
        println!("-- shininess_strength: {}", props.shininess_strength);
        println!("-- reflectivity: {}", props.reflectivity);
        println!("-- refracti: {}", props.refracti);
        println!("-- color_diffuse: {:?}", props.color_diffuse);
        println!("-- color_ambient: {:?}", props.color_ambient);
        println!("-- color_specular: {:?}", props.color_specular);
        println!("-- color_emissive: {:?}", props.color_emissive);
        println!("-- color_transparent: {:?}", props.color_transparent);
        println!("-- color_reflective: {:?}", props.color_reflective);

        let tex_tys = vec![
            ai::TextureType::None, 
            ai::TextureType::Diffuse,
            ai::TextureType::Specular,
            ai::TextureType::Ambient,
            ai::TextureType::Emissive,
            ai::TextureType::Height,
            ai::TextureType::Normals,
            ai::TextureType::Shininess,
            ai::TextureType::Opacity,
            ai::TextureType::Displacement,
            ai::TextureType::Lightmap,
            ai::TextureType::Reflection,
        ];
        for tex_ty in tex_tys {
            for idx2 in 0..mat.count_texture_properties(tex_ty) {
                println!("-| Texture ({:?}) #{}", tex_ty, idx2);
                println!("-- Properties: {:#?}", mat.texture_properties(tex_ty, idx2).unwrap());
            }
        }
    }
   
    println!("\n=== Textures ===");
    for (idx, tex) in scene.textures().iter().enumerate() {
        println!("| Texture #{}", idx);
        println!("- Bytes: {}", tex.as_bytes().len());
        println!("- Format Hint: {:?}", tex.format_hint());
        tex.as_texels().map(|(w, h, _)| {
            println!("- Texels: {}x{}", w, h);
        });
    }
}
