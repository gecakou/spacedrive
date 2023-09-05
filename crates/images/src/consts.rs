/// The size of 1MiB in bytes
const MIB: u64 = 1_048_576;

pub const HEIF_EXTENSIONS: [&str; 7] = ["heif", "heifs", "heic", "heics", "avif", "avci", "avcs"];

/// The maximum file size that an image can be in order to have a thumbnail generated.
///
/// This value is in MiB.
pub const HEIF_MAXIMUM_FILE_SIZE: u64 = MIB * 24;

pub const SVG_EXTENSIONS: [&str; 2] = ["svg", "svgz"];
/// The maximum file size that an image can be in order to have a thumbnail generated.
///
/// This value is in MiB.
pub const SVG_MAXIMUM_FILE_SIZE: u64 = MIB * 24;

/// This is not all `RAW` extensions, but a subset of the most common ones,
/// and the ones that the `rawloader` crate are most likely to support.
pub const RAW_EXTENSIONS: [&str; 13] = [
	"arw", "crw", "cr2", "cr3", "dng", "mdc", "mrw", "orf", "r3d", "sr2", "srf", "srw", "raw",
];

/// The maximum file size that an image can be in order to have a thumbnail generated.
///
/// This value is in MiB.
pub const RAW_MAXIMUM_FILE_SIZE: u64 = MIB * 64;

/// The maximum file size that an image can be in order to have a thumbnail generated.
///
/// This value is in MiB.
pub const GENERIC_MAXIMUM_FILE_SIZE: u64 = MIB * 32;

// This is the *full* list of RAW extensions, I'm not sure which we're 100% going to
// be able to support so I chose the most common ones
// pub const RAW_EXTENSIONS: [&str; 43] = [
// 	"3fr", "ari", "arw", "bay", "braw", "crw", "cr2", "cr3", "cap", "data", "dcs", "dcr", "dng",
// 	"drf", "eip", "erf", "fff", "gpr", "iiq", "k25", "kdc", "mdc", "mef", "mos", "mrw", "nef",
// 	"nrw", "obm", "orf", "pef", "ptx", "pxn", "r3d", "raf", "raw", "rwl", "rw2", "rwz", "sr2",
// 	"srf", "srw", "tif", "x3f",
// ];
