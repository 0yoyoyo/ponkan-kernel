extern "C" {
    fn set_cr3(value: u64);
}

const PAGE_DIRECTORY_COUNT: usize = 64;

const PAGE_SIZE_4K: u64 = 4096;
const PAGE_SIZE_2M: u64 = 512 * PAGE_SIZE_4K;
const PAGE_SIZE_1G: u64 = 512 * PAGE_SIZE_2M;

#[repr(align(4096))]
struct AlignedTable([u64; 512]);

#[repr(align(4096))]
struct AlignedDirectory([[u64; 512]; PAGE_DIRECTORY_COUNT]);

static mut PML4_TABLE: AlignedTable = AlignedTable([0; 512]);
static mut PDP_TABLE: AlignedTable = AlignedTable([0; 512]);
static mut PAGE_DIRECTORY: AlignedDirectory
    = AlignedDirectory([[0; 512]; PAGE_DIRECTORY_COUNT]);

pub fn setup_identity_page_table() {
    unsafe {
        PML4_TABLE.0[0] = &PDP_TABLE as *const _ as u64 | 0x003;
        for (i, pd) in PAGE_DIRECTORY.0.iter_mut().enumerate() {
            PDP_TABLE.0[i] = &pd[0] as *const _ as u64 | 0x003;
            for j in 0..512 {
                PAGE_DIRECTORY.0[i][j] =
                    (i as u64 * PAGE_SIZE_1G + j as u64 * PAGE_SIZE_2M) | 0x083;
            }
        }

        set_cr3(&PML4_TABLE as *const _ as u64);
    }
}
