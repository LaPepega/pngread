// TODO: At least *some* error handling

#[derive(Debug, Clone)]
pub struct Chunk {
    pub length: u32,
    pub chunk_type: Vec<u8>,
    pub data: Vec<u8>,
    pub crc: Vec<u8>,
}

impl Chunk {
    pub fn new(length: u32, chunk_type: Vec<u8>, data: Vec<u8>, crc: Vec<u8>) -> Self {
        Self {
            length,
            chunk_type,
            data,
            crc,
        }
    }

    // Chunk's type ASCII representation
    pub fn type_str(&self) -> String {
        self.chunk_type
            .iter()
            .map(|b| char::from_u32(b.clone() as u32).expect("Invalid character in chunk's name"))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct PNG {
    pub header: PNGHeader,
    pub chunks: Vec<Chunk>,
    pub image: RGBImage,
    pub palette: Option<Vec<RGB>>,
}

pub type RGB = (u8, u8, u8);

#[derive(Debug, Clone)]
pub struct RGBImage(pub Vec<Vec<RGB>>);

// FIXME: currently panics if bit_depth != 8
// FIXME: filter-type bytes are being read as part of the image
impl RGBImage {
    // FIXME: broken
    pub fn from_grayscale_idat(idat: Chunk, bit_depth: u8, width: u32) -> Self {
        let inflated = inflate::inflate_bytes_zlib(&idat.data).expect("Inflation failed");

        let img: Vec<Vec<RGB>> = inflated
            .chunks(width as usize)
            .map(|line| line.iter().map(|pixel| (*pixel, *pixel, *pixel)).collect())
            .collect();

        Self(img)
    }

    pub fn from_rgb_idat(idat: Chunk, bit_depth: u8, width: u32) -> Self {
        for b in idat.data {}
        todo!()
    }

    pub fn from_palette_idat(
        idat: Chunk,
        bit_depth: u8,
        width: u32,
        palette: Option<Vec<RGB>>,
    ) -> Self {
        let plte = palette.expect("Image has no palette");

        // "PNG compression method 0 (the only compression method presently defined for PNG) specifies deflate/inflate compression"
        let inflated = inflate::inflate_bytes_zlib(&idat.data).expect("Inflation failed");

        // Go over each byte in the idat matching it's value to palette
        let img: Vec<Vec<RGB>> = inflated
            .chunks(width as usize + 1)
            .map(|line| {
                line.iter()
                    // Skip filter-type
                    .skip(2)
                    .map(|pixel| plte[*pixel as usize])
                    .collect()
            })
            .collect();

        Self(img)
    }

    pub fn from_alpha_grayscale_idat(idat: Chunk, bit_depth: u8, width: u32) -> Self {
        for b in idat.data {}
        todo!()
    }

    pub fn from_alpha_rgb_idat(idat: Chunk, bit_depth: u8, width: u32) -> Self {
        for b in idat.data {}
        todo!()
    }
}

impl PNG {
    pub fn new(
        header: PNGHeader,
        chunks: Vec<Chunk>,
        palette: Option<Vec<RGB>>,
        image: RGBImage,
    ) -> Self {
        Self {
            header,
            chunks,
            palette,
            image,
        }
    }

    //TODO: Does too much, delegate
    pub fn from_bytes(mut bytes: Vec<u8>) -> Self {
        let mut chunks = Vec::<Chunk>::new();

        while !bytes.is_empty() {
            let chunk_len =
                u32::from_be_bytes(bytes.drain(..4).collect::<Vec<u8>>().try_into().unwrap());
            let chunk_type: Vec<u8> = bytes.drain(..4).collect();
            let chunk_data: Vec<u8> = bytes.drain(..chunk_len as usize).collect();
            let chunk_crc: Vec<u8> = bytes.drain(..4).collect();
            chunks.push(Chunk::new(chunk_len, chunk_type, chunk_data, chunk_crc));
        }

        // Assuming IHDR chunk always comes first
        let header: PNGHeader = chunks[0].clone().into();

        let palette: Option<Vec<RGB>> = match chunks.iter().find(|c| c.type_str() == "PLTE") {
            // A chunk length not divisible by 3 is an error.
            Some(plte_c) => Some(plte_c.data.chunks(3).map(|c| (c[0], c[1], c[2])).collect()),
            None => None,
        };

        let idat = chunks
            .iter()
            .find(|c| c.type_str() == "IDAT")
            .expect("Image has no IDAT chunk")
            .clone();

        let image = match header.color_type {
            0 => RGBImage::from_grayscale_idat(idat, header.bit_depth, header.width),
            2 => RGBImage::from_rgb_idat(idat, header.bit_depth, header.width),
            3 => RGBImage::from_palette_idat(idat, header.bit_depth, header.width, palette.clone()),
            4 => RGBImage::from_alpha_grayscale_idat(idat, header.bit_depth, header.width),
            6 => RGBImage::from_alpha_rgb_idat(idat, header.bit_depth, header.width),
            _ => panic!("Invalid color type"),
        };

        Self {
            header,
            chunks,
            palette,
            image,
        }
    }

    pub fn from_chunks(chunks: Vec<Chunk>) -> Self {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct PNGHeader {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

// Reads header data from IHDR chunk
impl From<Chunk> for PNGHeader {
    fn from(mut value: Chunk) -> Self {
        let width = u32::from_be_bytes(
            value
                .data
                .drain(..4)
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap(),
        );

        let height = u32::from_be_bytes(
            value
                .data
                .drain(..4)
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap(),
        );

        let bit_depth: u8 = value.data.drain(..1).nth(0).unwrap();
        let color_type: u8 = value.data.drain(..1).nth(0).unwrap();
        let compression_method: u8 = value.data.drain(..1).nth(0).unwrap();
        let filter_method: u8 = value.data.drain(..1).nth(0).unwrap();
        let interlace_method: u8 = value.data.drain(..1).nth(0).unwrap();

        PNGHeader {
            width,
            height,
            bit_depth,
            color_type,
            compression_method,
            filter_method,
            interlace_method,
        }
    }
}
