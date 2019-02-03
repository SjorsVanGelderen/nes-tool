use cgmath::Vector2;

use crate::surface::Surface;

struct Palette {
    // color_indices: [u32; 26],
}

impl Palette {
    fn zero() -> Palette {
        Palette {
            // color_indices = 
        }
    }

    // fn set_color_index(&self, which: i32, color_index: i32) -> PaletteData {
    //     let mut color_indices = self.color_indices;
    //     color_indices[which] = color_index;

    //     Palette { color_indices: color_indices, ..self }
    // }
}

pub fn surface_zero() -> Surface {
    Surface::zero(Vector2::new(0.0, 0.0), Vector2::new(16.0, 4.0))
}

// let full_palette: [u8; 192] = [ 
//     101u8, 101u8, 101u8,
//       3u8,  47u8, 103u8,
//      21u8,  35u8, 125u8,
//      60u8,  26u8, 122u8,
//      95u8,  18u8,  97u8,
//     114u8,  14u8,  55u8,
//     112u8,  16u8,  13u8,
//      89u8,  26u8,   5u8,
//      52u8,  40u8,   3u8,
//      13u8,  51u8,   3u8,
//       3u8,  59u8,   4u8,
//       4u8,  60u8,  19u8,
//       3u8,  56u8,  63u8,
//       0u8,   0u8,   0u8,
//       0u8,   0u8,   0u8,
//       0u8,   0u8,   0u8,
  
//     174u8, 174u8, 174u8,
//      15u8,  99u8, 179u8,
//      64u8,  81u8, 208u8,
//     120u8,  65u8, 204u8,
//     167u8,  54u8, 169u8,
//     192u8,  52u8, 112u8,
//     189u8,  60u8,  48u8,
//     159u8,  74u8,   0u8,
//     109u8,  92u8,   0u8,
//      54u8, 109u8,   0u8,
//       7u8, 119u8,   4u8,
//       0u8, 121u8,  61u8,
//       0u8, 114u8, 125u8,
//       0u8,   0u8,   0u8,
//       0u8,   0u8,   0u8,
//       0u8,   0u8,   0u8,

//     254u8, 254u8, 255u8,
//      93u8, 179u8, 255u8,
//     143u8, 161u8, 255u8,
//     200u8, 144u8, 255u8,
//     247u8, 133u8, 250u8,
//     255u8, 131u8, 192u8,
//     255u8, 138u8, 127u8,
//     239u8, 154u8,  73u8,
//     189u8, 172u8,  44u8,
//     133u8, 188u8,  47u8,
//      85u8, 199u8,  83u8,
//      60u8, 201u8, 140u8,
//      62u8, 194u8, 205u8,
//      78u8,  78u8,  78u8,
//       0u8,   0u8,   0u8,
//       0u8,   0u8,   0u8,

//     254u8, 254u8, 255u8,
//     188u8, 223u8, 255u8,
//     209u8, 216u8, 255u8,
//     232u8, 209u8, 255u8,
//     251u8, 205u8, 253u8,
//     255u8, 204u8, 229u8,
//     255u8, 207u8, 202u8,
//     248u8, 213u8, 180u8,
//     228u8, 220u8, 168u8,
//     204u8, 227u8, 169u8,
//     185u8, 232u8, 184u8,
//     174u8, 232u8, 208u8,
//     175u8, 229u8, 234u8,
//     182u8, 182u8, 182u8,
//       0u8,   0u8,   0u8,
//       0u8,   0u8,   0u8,
//   ];
