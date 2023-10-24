use alloc::sync::Arc;
use limine::ModuleRequest;

pub static MODULE_REQUEST: ModuleRequest = ModuleRequest::new(0);

pub fn init() {
    // TODO: Put this stuff in another file?
    if MODULE_REQUEST.get_response().get().is_none() {
        crate::log_error!("Module request in none!");
        return;
    }
    let module_response = MODULE_REQUEST.get_response().get().unwrap();

    let mut initramfs = None;

    let module_name = "initramfs.img";

    for module in module_response.modules() {
        let c_path = module.path.to_str();
        if c_path.is_none() {
            continue;
        }

        if !c_path.unwrap().to_str().unwrap().contains(module_name) {
            continue;
        }

        initramfs = Some(module);
    }

    if initramfs.is_none() {
        crate::log_error!("Initramfs was not found!");
        return;
    }
    let initramfs = initramfs.unwrap();

    crate::println!("Initramfs is located at: {:#018X?}", unsafe {
        initramfs.base.as_ptr().unwrap()
            ..initramfs
                .base
                .as_ptr()
                .unwrap()
                .add(initramfs.length as usize)
    });

    init_fs(initramfs.base.as_ptr().unwrap());
}

#[derive(Clone, Copy, Debug)]
#[repr(u16)]
enum SquashfsCompressionType {
    GZIP = 1,
    LZMA = 2,
    LZO = 3,
    XZ = 4,
    LZ4 = 5,
    ZSTD = 6,
}

impl Into<SquashfsCompressionType> for u16 {
    fn into(self) -> SquashfsCompressionType {
        match self {
            1 => SquashfsCompressionType::GZIP,
            2 => SquashfsCompressionType::LZMA,
            3 => SquashfsCompressionType::LZO,
            4 => SquashfsCompressionType::XZ,
            5 => SquashfsCompressionType::LZ4,
            6 => SquashfsCompressionType::ZSTD,
            _ => panic!("Unexpected Squashfs compression type!"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct SquashfsSuperblock {
    magic: u32,
    inode_count: u32,
    mod_time: u32,
    block_size: u32,
    frag_count: u32,
    compressor: SquashfsCompressionType,
    block_log: u16,
    flags: u16,
    id_count: u16,
    ver_major: u16,
    ver_minor: u16,
    root_inode: u64,
    bytes_used: u64,
    id_table: u64,
    xattr_table: u64,
    inode_table: u64,
    dir_table: u64,
    frag_table: u64,
    export_table: u64,
}

impl SquashfsSuperblock {
    fn new(bytes: Arc<[u8]>) -> Self {
        return Self {
            magic: u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            inode_count: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            mod_time: u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
            block_size: u32::from_le_bytes(bytes[12..16].try_into().unwrap()),
            frag_count: u32::from_le_bytes(bytes[16..20].try_into().unwrap()),
            compressor: u16::from_le_bytes(bytes[20..22].try_into().unwrap()).into(),
            block_log: u16::from_le_bytes(bytes[22..24].try_into().unwrap()),
            flags: u16::from_le_bytes(bytes[24..26].try_into().unwrap()),
            id_count: u16::from_le_bytes(bytes[26..28].try_into().unwrap()),
            ver_major: u16::from_le_bytes(bytes[28..30].try_into().unwrap()),
            ver_minor: u16::from_le_bytes(bytes[30..32].try_into().unwrap()),
            root_inode: u64::from_le_bytes(bytes[32..40].try_into().unwrap()),
            bytes_used: u64::from_le_bytes(bytes[40..48].try_into().unwrap()),
            id_table: u64::from_le_bytes(bytes[48..56].try_into().unwrap()),
            xattr_table: u64::from_le_bytes(bytes[56..64].try_into().unwrap()),
            inode_table: u64::from_le_bytes(bytes[64..72].try_into().unwrap()),
            dir_table: u64::from_le_bytes(bytes[72..80].try_into().unwrap()),
            frag_table: u64::from_le_bytes(bytes[80..88].try_into().unwrap()),
            export_table: u64::from_le_bytes(bytes[88..96].try_into().unwrap()),
        };
    }
}

fn init_fs(initramfs: *mut u8) {
    crate::println!("Parsing initramfs fs at {:p}", initramfs);
    let mut superblock_bytes = [0u8; core::mem::size_of::<SquashfsSuperblock>()];
    unsafe {
        initramfs.copy_to(
            superblock_bytes.as_mut_ptr(),
            core::mem::size_of::<SquashfsSuperblock>(),
        )
    }
    let superblock = SquashfsSuperblock::new(Arc::from(superblock_bytes));

    crate::println!("{:X?}", superblock);
}
