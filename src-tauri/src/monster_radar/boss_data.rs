//! Boss and SuperUnique Monster Class IDs for Median XL.
//! Extracted directly from local unpacked data:
//! `superuniques.bin` (90 superunique rows) and `monstats.bin` (Act Bosses).

/// Sorted list of all fixed Boss and SuperUnique monster Class IDs.
pub const BOSS_CLASS_IDS: &[u16] = &[
    45,  // Griswold (格里斯瓦德)
    156, // Andariel (安达利尔)
    211, // Duriel (都瑞尔)
    229, // Radament (罗达门特)
    242, // Mephisto (墨菲斯托)
    243, // Diablo (迪亚波罗)
    250, // Summoner (召唤者)
    284, // Smith (铁匠)
    365, // Izual (衣卒尔)
    402, // Ancient Talic (塔力克)
    409, // Hephasto (赫法斯托)
    526, // Shenk (督军山克)
    540, // Nihlathak (尼拉塞克)
    541, // Talic (塔力克)
    542, // Madawc (马道克)
    544, // Baal (巴尔)
    935, 945, 947, 948, 949, 950, 951, 952, 953, 954, 955, 956, 957, 958, 959, 960, 961, 962, 963,
    964, 965, 966, 967, 968, 969, 970, 971, 972, 973, 974, 975, 976, 977, 978, 979, 980, 981, 982,
    983, 984, 985, 986, 987, 988, 989, 990, 991, 992, 993, 994, 995, 996, 997, 998, 999, 1000,
    1001, 1002, 1003, 1004, 1005, 1006, 2823, // Bishibosh (毕须博须)
    2839, // Corpsefire (尸体发火)
    2841, // Rakanishu (拉卡尼休)
    2881, // Treehead Woodfist (树头木拳)
];

/// Fast binary search check if a class ID is a known Boss / SuperUnique.
pub fn is_boss_class(class_id: u16) -> bool {
    BOSS_CLASS_IDS.binary_search(&class_id).is_ok()
}

/// Generates a 4096-byte lookup table where `table[class_id] == 1` for bosses, `0` otherwise.
pub fn generate_boss_lookup_table() -> [u8; 4096] {
    let mut table = [0u8; 4096];
    for &id in BOSS_CLASS_IDS {
        if (id as usize) < table.len() {
            table[id as usize] = 1;
        }
    }
    table
}
