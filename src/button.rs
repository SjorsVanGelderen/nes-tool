// let (texture, tex_future) = {
//     let image = image::load_from_memory_with_format(
//         include_bytes!("pencil.png"),
//         ImageFormat::PNG
//     ).unwrap().to_rgba();

//     let image_data = image.into_raw().clone();

//     ImmutableImage::from_iter(
//         image_data.iter().cloned(),
//         Dimensions::Dim2d { width: 256, height: 256 },
//         Format::R8G8B8A8Srgb,
//         queue.clone()
//     ).unwrap()
// };