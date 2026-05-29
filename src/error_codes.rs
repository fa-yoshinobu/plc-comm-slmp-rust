//! SLMP end-code names and messages.

/// Language selector for SLMP end-code messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlmpEndCodeLanguage {
    /// English.
    English,
    /// Japanese.
    Japanese,
}

/// Return a compact code-derived diagnostic label for an SLMP end code.
///
/// Use numeric codes or category helpers for branching.
pub fn end_code_name(end_code: u16) -> &'static str {
    match end_code {
        0x1080 => "slmp_end_code_1080",
        0x1120 => "slmp_end_code_1120",
        0x1124 => "slmp_end_code_1124",
        0x1128 => "slmp_end_code_1128",
        0x1129 => "slmp_end_code_1129",
        0x112C => "slmp_end_code_112c",
        0x112D => "slmp_end_code_112d",
        0x112E => "slmp_end_code_112e",
        0x1133 => "slmp_end_code_1133",
        0x1134 => "slmp_end_code_1134",
        0x1152 => "slmp_end_code_1152",
        0x1155 => "slmp_end_code_1155",
        0x1157 => "slmp_end_code_1157",
        0x1158 => "slmp_end_code_1158",
        0x1165 => "slmp_end_code_1165",
        0x1166 => "slmp_end_code_1166",
        0x1167 => "slmp_end_code_1167",
        0x1180 => "slmp_end_code_1180",
        0x1800 => "slmp_end_code_1800",
        0x1801 => "slmp_end_code_1801",
        0x1811 => "slmp_end_code_1811",
        0x1830 => "slmp_end_code_1830",
        0x1845 => "slmp_end_code_1845",
        0x1860 => "slmp_end_code_1860",
        0x1D01 => "slmp_end_code_1d01",
        0x1D10 => "slmp_end_code_1d10",
        0x1D20 => "slmp_end_code_1d20",
        0x1F07 => "slmp_end_code_1f07",
        0x20E0 => "slmp_end_code_20e0",
        0x2160 => "slmp_end_code_2160",
        0x2220 => "slmp_end_code_2220",
        0x2221 => "slmp_end_code_2221",
        0x2250 => "slmp_end_code_2250",
        0x24C0 => "slmp_end_code_24c0",
        0x24C1 => "slmp_end_code_24c1",
        0x24C2 => "slmp_end_code_24c2",
        0x24C3 => "slmp_end_code_24c3",
        0x24C6 => "slmp_end_code_24c6",
        0x2600 => "slmp_end_code_2600",
        0x2610 => "slmp_end_code_2610",
        0x3000 => "slmp_end_code_3000",
        0x3001 => "slmp_end_code_3001",
        0x3004 => "slmp_end_code_3004",
        0x3005 => "slmp_end_code_3005",
        0x3006 => "slmp_end_code_3006",
        0x3007 => "slmp_end_code_3007",
        0x3008 => "slmp_end_code_3008",
        0x3019 => "slmp_end_code_3019",
        0x301A => "slmp_end_code_301a",
        0x301B => "slmp_end_code_301b",
        0x301C => "slmp_end_code_301c",
        0x301D => "slmp_end_code_301d",
        0x301E => "slmp_end_code_301e",
        0x301F => "slmp_end_code_301f",
        0x3020 => "slmp_end_code_3020",
        0x3022 => "slmp_end_code_3022",
        0x3023 => "slmp_end_code_3023",
        0x3026 => "slmp_end_code_3026",
        0x3040 => "slmp_end_code_3040",
        0x3060 => "slmp_end_code_3060",
        0x31D0 => "slmp_end_code_31d0",
        0x3600 => "slmp_end_code_3600",
        0x3601 => "slmp_end_code_3601",
        0x3602 => "slmp_end_code_3602",
        0x3C00 => "slmp_end_code_3c00",
        0x3C01 => "slmp_end_code_3c01",
        0x3C02 => "slmp_end_code_3c02",
        0x3C03 => "slmp_end_code_3c03",
        0x3C0F => "slmp_end_code_3c0f",
        0x3C10 => "slmp_end_code_3c10",
        0x3C11 => "slmp_end_code_3c11",
        0x3C13 => "slmp_end_code_3c13",
        0x3C14 => "slmp_end_code_3c14",
        0x3C2F => "slmp_end_code_3c2f",
        0x3E00 => "slmp_end_code_3e00",
        0x3E01 => "slmp_end_code_3e01",
        0xC001 => "slmp_end_code_c001",
        0xC012 => "slmp_end_code_c012",
        0xC013 => "slmp_end_code_c013",
        0xC015 => "slmp_end_code_c015",
        0xC016 => "slmp_end_code_c016",
        0xC018 => "slmp_end_code_c018",
        0xC020 => "slmp_end_code_c020",
        0xC021 => "slmp_end_code_c021",
        0xC022 => "slmp_end_code_c022",
        0xC024 => "slmp_end_code_c024",
        0xC025 => "slmp_end_code_c025",
        0xC026 => "slmp_end_code_c026",
        0xC027 => "slmp_end_code_c027",
        0xC028 => "slmp_end_code_c028",
        0xC029 => "slmp_end_code_c029",
        0xC035 => "slmp_end_code_c035",
        0xC040 => "slmp_end_code_c040",
        0xC050 => "slmp_end_code_c050",
        0xC051 => "slmp_end_code_c051",
        0xC052 => "slmp_end_code_c052",
        0xC053 => "slmp_end_code_c053",
        0xC054 => "slmp_end_code_c054",
        0xC055 => "slmp_end_code_c055",
        0xC056 => "slmp_end_code_c056",
        0xC057 => "slmp_end_code_c057",
        0xC058 => "slmp_end_code_c058",
        0xC059 => "slmp_end_code_c059",
        0xC05A => "slmp_end_code_c05a",
        0xC05B => "slmp_end_code_c05b",
        0xC05C => "slmp_end_code_c05c",
        0xC05D => "slmp_end_code_c05d",
        0xC05E => "slmp_end_code_c05e",
        0xC05F => "slmp_end_code_c05f",
        0xC060 => "slmp_end_code_c060",
        0xC061 => "slmp_end_code_c061",
        0xC070 => "slmp_end_code_c070",
        0xC071 => "slmp_end_code_c071",
        0xC072 => "slmp_end_code_c072",
        0xC073 => "slmp_end_code_c073",
        0xC075 => "slmp_end_code_c075",
        0xC081 => "slmp_end_code_c081",
        0xC083 => "slmp_end_code_c083",
        0xC084 => "slmp_end_code_c084",
        0xC085 => "slmp_end_code_c085",
        0xC0B2 => "slmp_end_code_c0b2",
        0xC0B3 => "slmp_end_code_c0b3",
        0xC0B6 => "slmp_end_code_c0b6",
        0xC0BA => "slmp_end_code_c0ba",
        0xC0C4 => "slmp_end_code_c0c4",
        0xC0D0 => "slmp_end_code_c0d0",
        0xC0D1 => "slmp_end_code_c0d1",
        0xC0D3 => "slmp_end_code_c0d3",
        0xC0D4 => "slmp_end_code_c0d4",
        0xC0D5 => "slmp_end_code_c0d5",
        0xC0D6 => "slmp_end_code_c0d6",
        0xC0D7 => "slmp_end_code_c0d7",
        0xC0D8 => "slmp_end_code_c0d8",
        0xC0D9 => "slmp_end_code_c0d9",
        0xC0DA => "slmp_end_code_c0da",
        0xC0DB => "slmp_end_code_c0db",
        0xC0DE => "slmp_end_code_c0de",
        0xC101 => "slmp_end_code_c101",
        0xC1A2 => "slmp_end_code_c1a2",
        0xC1A4 => "slmp_end_code_c1a4",
        0xC1A5 => "slmp_end_code_c1a5",
        0xC1A6 => "slmp_end_code_c1a6",
        0xC1A7 => "slmp_end_code_c1a7",
        0xC1A8 => "slmp_end_code_c1a8",
        0xC1A9 => "slmp_end_code_c1a9",
        0xC1AA => "slmp_end_code_c1aa",
        0xC1AC => "slmp_end_code_c1ac",
        0xC1AD => "slmp_end_code_c1ad",
        0xC1AF => "slmp_end_code_c1af",
        0xC1B0 => "slmp_end_code_c1b0",
        0xC1B1 => "slmp_end_code_c1b1",
        0xC1B2 => "slmp_end_code_c1b2",
        0xC1B3 => "slmp_end_code_c1b3",
        0xC1B4 => "slmp_end_code_c1b4",
        0xC1B8 => "slmp_end_code_c1b8",
        0xC1B9 => "slmp_end_code_c1b9",
        0xC1BA => "slmp_end_code_c1ba",
        0xC1BB => "slmp_end_code_c1bb",
        0xC1BC => "slmp_end_code_c1bc",
        0xC1BD => "slmp_end_code_c1bd",
        0xC1BE => "slmp_end_code_c1be",
        0xC1BF => "slmp_end_code_c1bf",
        0xC1C0 => "slmp_end_code_c1c0",
        0xC1C1 => "slmp_end_code_c1c1",
        0xC1C2 => "slmp_end_code_c1c2",
        0xC1C4 => "slmp_end_code_c1c4",
        0xC1C5 => "slmp_end_code_c1c5",
        0xC1C6 => "slmp_end_code_c1c6",
        0xC1C7 => "slmp_end_code_c1c7",
        0xC1C8 => "slmp_end_code_c1c8",
        0xC1C9 => "slmp_end_code_c1c9",
        0xC1CA => "slmp_end_code_c1ca",
        0xC1CB => "slmp_end_code_c1cb",
        0xC1CC => "slmp_end_code_c1cc",
        0xC1CD => "slmp_end_code_c1cd",
        0xC1D0 => "slmp_end_code_c1d0",
        0xC1D2 => "slmp_end_code_c1d2",
        0xC1D3 => "slmp_end_code_c1d3",
        0xC200 => "slmp_end_code_c200",
        0xC201 => "slmp_end_code_c201",
        0xC202 => "slmp_end_code_c202",
        0xC203 => "slmp_end_code_c203",
        0xC204 => "slmp_end_code_c204",
        0xC205 => "slmp_end_code_c205",
        0xC207 => "slmp_end_code_c207",
        0xC208 => "slmp_end_code_c208",
        0xC400 => "slmp_end_code_c400",
        0xC401 => "slmp_end_code_c401",
        0xC402 => "slmp_end_code_c402",
        0xC403 => "slmp_end_code_c403",
        0xC404 => "slmp_end_code_c404",
        0xC405 => "slmp_end_code_c405",
        0xC406 => "slmp_end_code_c406",
        0xC407 => "slmp_end_code_c407",
        0xC408 => "slmp_end_code_c408",
        0xC410 => "slmp_end_code_c410",
        0xC412 => "slmp_end_code_c412",
        0xC413 => "slmp_end_code_c413",
        0xC414 => "slmp_end_code_c414",
        0xC417 => "slmp_end_code_c417",
        0xC420 => "slmp_end_code_c420",
        0xC421 => "slmp_end_code_c421",
        0xC430 => "slmp_end_code_c430",
        0xC431 => "slmp_end_code_c431",
        0xC440 => "slmp_end_code_c440",
        0xC441 => "slmp_end_code_c441",
        0xC442 => "slmp_end_code_c442",
        0xC443 => "slmp_end_code_c443",
        0xC444 => "slmp_end_code_c444",
        0xC445 => "slmp_end_code_c445",
        0xC446 => "slmp_end_code_c446",
        0xC447 => "slmp_end_code_c447",
        0xC448 => "slmp_end_code_c448",
        0xC449 => "slmp_end_code_c449",
        0xC44A => "slmp_end_code_c44a",
        0xC44B => "slmp_end_code_c44b",
        0xC44C => "slmp_end_code_c44c",
        0xC44D => "slmp_end_code_c44d",
        0xC44E => "slmp_end_code_c44e",
        0xC44F => "slmp_end_code_c44f",
        0xC610 => "slmp_end_code_c610",
        0xC611 => "slmp_end_code_c611",
        0xC612 => "slmp_end_code_c612",
        0xC613 => "slmp_end_code_c613",
        0xC614 => "slmp_end_code_c614",
        0xC615 => "slmp_end_code_c615",
        0xC616 => "slmp_end_code_c616",
        0xC617 => "slmp_end_code_c617",
        0xC618 => "slmp_end_code_c618",
        0xC619 => "slmp_end_code_c619",
        0xC620 => "slmp_end_code_c620",
        0xC621 => "slmp_end_code_c621",
        0xC622 => "slmp_end_code_c622",
        0xC623 => "slmp_end_code_c623",
        0xC700 => "slmp_end_code_c700",
        0xC701 => "slmp_end_code_c701",
        0xC702 => "slmp_end_code_c702",
        0xC703 => "slmp_end_code_c703",
        0xC704 => "slmp_end_code_c704",
        0xC705 => "slmp_end_code_c705",
        0xC706 => "slmp_end_code_c706",
        0xC707 => "slmp_end_code_c707",
        0xC708 => "slmp_end_code_c708",
        0xC709 => "slmp_end_code_c709",
        0xC810 => "slmp_end_code_c810",
        0xC811 => "slmp_end_code_c811",
        0xC812 => "slmp_end_code_c812",
        0xC813 => "slmp_end_code_c813",
        0xC814 => "slmp_end_code_c814",
        0xC815 => "slmp_end_code_c815",
        0xC816 => "slmp_end_code_c816",
        0xC840 => "slmp_end_code_c840",
        0xC842 => "slmp_end_code_c842",
        0xC843 => "slmp_end_code_c843",
        0xC844 => "slmp_end_code_c844",
        0xC860 => "slmp_end_code_c860",
        0xC861 => "slmp_end_code_c861",
        0xC862 => "slmp_end_code_c862",
        0xC863 => "slmp_end_code_c863",
        0xC864 => "slmp_end_code_c864",
        0xC865 => "slmp_end_code_c865",
        0xC866 => "slmp_end_code_c866",
        0xC867 => "slmp_end_code_c867",
        0xC868 => "slmp_end_code_c868",
        0xC869 => "slmp_end_code_c869",
        0xC86A => "slmp_end_code_c86a",
        0xC86B => "slmp_end_code_c86b",
        0xC86C => "slmp_end_code_c86c",
        0xCEE0 => "slmp_end_code_cee0",
        0xCEE1 => "slmp_end_code_cee1",
        0xCEE2 => "slmp_end_code_cee2",
        0xCF10 => "slmp_end_code_cf10",
        0xCF20 => "slmp_end_code_cf20",
        0xCF30 => "slmp_end_code_cf30",
        0xCF31 => "slmp_end_code_cf31",
        0xCF70 => "slmp_end_code_cf70",
        0xCF71 => "slmp_end_code_cf71",
        0xCF80 => "slmp_end_code_cf80",
        0xCF81 => "slmp_end_code_cf81",
        0xCF82 => "slmp_end_code_cf82",
        0xCF83 => "slmp_end_code_cf83",
        0xCF84 => "slmp_end_code_cf84",
        0xCF85 => "slmp_end_code_cf85",
        0xCF8A => "slmp_end_code_cf8a",
        0xCF8C => "slmp_end_code_cf8c",
        0xCFB0 => "slmp_end_code_cfb0",
        0xCFB1 => "slmp_end_code_cfb1",
        0xCFB2 => "slmp_end_code_cfb2",
        0xCFB3 => "slmp_end_code_cfb3",
        0xCFB4 => "slmp_end_code_cfb4",
        0xCFB5 => "slmp_end_code_cfb5",
        0xCFBD => "slmp_end_code_cfbd",
        0xCFBE => "slmp_end_code_cfbe",
        0xCFBF => "slmp_end_code_cfbf",
        0xD000 => "slmp_end_code_d000",
        0xD038 => "slmp_end_code_d038",
        0xD039 => "slmp_end_code_d039",
        0xD03B => "slmp_end_code_d03b",
        0xD041 => "slmp_end_code_d041",
        0xD080 => "slmp_end_code_d080",
        0xD081 => "slmp_end_code_d081",
        0xD082 => "slmp_end_code_d082",
        0xD083 => "slmp_end_code_d083",
        0xD0A0 => "slmp_end_code_d0a0",
        0xD0A1 => "slmp_end_code_d0a1",
        0xD0A2 => "slmp_end_code_d0a2",
        0xD0A3 => "slmp_end_code_d0a3",
        0xD0A4 => "slmp_end_code_d0a4",
        0xD0A5 => "slmp_end_code_d0a5",
        0xD0A6 => "slmp_end_code_d0a6",
        0xD0C0 => "slmp_end_code_d0c0",
        0xD0C1 => "slmp_end_code_d0c1",
        0xD0C4 => "slmp_end_code_d0c4",
        0xD0C5 => "slmp_end_code_d0c5",
        0xD0D0 => "slmp_end_code_d0d0",
        0xD200 => "slmp_end_code_d200",
        0xD202 => "slmp_end_code_d202",
        0xD203 => "slmp_end_code_d203",
        0xD204 => "slmp_end_code_d204",
        0xD205 => "slmp_end_code_d205",
        0xD206 => "slmp_end_code_d206",
        0xD207 => "slmp_end_code_d207",
        0xD208 => "slmp_end_code_d208",
        0xD209 => "slmp_end_code_d209",
        0xD20A => "slmp_end_code_d20a",
        0xD20B => "slmp_end_code_d20b",
        0xD20C => "slmp_end_code_d20c",
        0xD20D => "slmp_end_code_d20d",
        0xD20E => "slmp_end_code_d20e",
        0xD20F => "slmp_end_code_d20f",
        0xD210 => "slmp_end_code_d210",
        0xD211 => "slmp_end_code_d211",
        0xD212 => "slmp_end_code_d212",
        0xD213 => "slmp_end_code_d213",
        0xD214 => "slmp_end_code_d214",
        0xD215 => "slmp_end_code_d215",
        0xD216 => "slmp_end_code_d216",
        0xD217 => "slmp_end_code_d217",
        0xD218 => "slmp_end_code_d218",
        0xD219 => "slmp_end_code_d219",
        0xD21A => "slmp_end_code_d21a",
        0xD21B => "slmp_end_code_d21b",
        0xD21C => "slmp_end_code_d21c",
        0xD21D => "slmp_end_code_d21d",
        0xD21E => "slmp_end_code_d21e",
        0xD21F => "slmp_end_code_d21f",
        0xD220 => "slmp_end_code_d220",
        0xD222 => "slmp_end_code_d222",
        0xD223 => "slmp_end_code_d223",
        0xD224 => "slmp_end_code_d224",
        0xD22E => "slmp_end_code_d22e",
        0xD22F => "slmp_end_code_d22f",
        0xD230 => "slmp_end_code_d230",
        0xD231 => "slmp_end_code_d231",
        0xD232 => "slmp_end_code_d232",
        0xD233 => "slmp_end_code_d233",
        0xD234 => "slmp_end_code_d234",
        0xD235 => "slmp_end_code_d235",
        0xD236 => "slmp_end_code_d236",
        0xD237 => "slmp_end_code_d237",
        0xD238 => "slmp_end_code_d238",
        0xD239 => "slmp_end_code_d239",
        0xD23A => "slmp_end_code_d23a",
        0xD23B => "slmp_end_code_d23b",
        0xD23C => "slmp_end_code_d23c",
        0xD23D => "slmp_end_code_d23d",
        0xD23E => "slmp_end_code_d23e",
        0xD240 => "slmp_end_code_d240",
        0xD241 => "slmp_end_code_d241",
        0xD242 => "slmp_end_code_d242",
        0xD243 => "slmp_end_code_d243",
        0xD244 => "slmp_end_code_d244",
        0xD245 => "slmp_end_code_d245",
        0xD246 => "slmp_end_code_d246",
        0xD247 => "slmp_end_code_d247",
        0xD249 => "slmp_end_code_d249",
        0xD24A => "slmp_end_code_d24a",
        0xD24B => "slmp_end_code_d24b",
        0xD24C => "slmp_end_code_d24c",
        0xD24D => "slmp_end_code_d24d",
        0xD24E => "slmp_end_code_d24e",
        0xD24F => "slmp_end_code_d24f",
        0xD251 => "slmp_end_code_d251",
        0xD252 => "slmp_end_code_d252",
        0xD253 => "slmp_end_code_d253",
        0xD254 => "slmp_end_code_d254",
        0xD255 => "slmp_end_code_d255",
        0xD256 => "slmp_end_code_d256",
        0xD257 => "slmp_end_code_d257",
        0xD258 => "slmp_end_code_d258",
        0xD25A => "slmp_end_code_d25a",
        0xD25B => "slmp_end_code_d25b",
        0xD25C => "slmp_end_code_d25c",
        0xD25D => "slmp_end_code_d25d",
        0xD25E => "slmp_end_code_d25e",
        0xD25F => "slmp_end_code_d25f",
        0xD260 => "slmp_end_code_d260",
        0xD261 => "slmp_end_code_d261",
        0xD262 => "slmp_end_code_d262",
        0xD263 => "slmp_end_code_d263",
        0xD264 => "slmp_end_code_d264",
        0xD265 => "slmp_end_code_d265",
        0xD266 => "slmp_end_code_d266",
        0xD267 => "slmp_end_code_d267",
        0xD268 => "slmp_end_code_d268",
        0xD269 => "slmp_end_code_d269",
        0xD26A => "slmp_end_code_d26a",
        0xD26B => "slmp_end_code_d26b",
        0xD26C => "slmp_end_code_d26c",
        0xD26F => "slmp_end_code_d26f",
        0xD270 => "slmp_end_code_d270",
        0xD271 => "slmp_end_code_d271",
        0xD272 => "slmp_end_code_d272",
        0xD273 => "slmp_end_code_d273",
        0xD274 => "slmp_end_code_d274",
        0xD275 => "slmp_end_code_d275",
        0xD276 => "slmp_end_code_d276",
        0xD277 => "slmp_end_code_d277",
        0xD278 => "slmp_end_code_d278",
        0xD279 => "slmp_end_code_d279",
        0xD27A => "slmp_end_code_d27a",
        0xD280 => "slmp_end_code_d280",
        0xD281 => "slmp_end_code_d281",
        0xD282 => "slmp_end_code_d282",
        0xD283 => "slmp_end_code_d283",
        0xD284 => "slmp_end_code_d284",
        0xD2A0 => "slmp_end_code_d2a0",
        0xD2A1 => "slmp_end_code_d2a1",
        0xD2A2 => "slmp_end_code_d2a2",
        0xD2A3 => "slmp_end_code_d2a3",
        0xD2A4 => "slmp_end_code_d2a4",
        0xD2A5 => "slmp_end_code_d2a5",
        0xD2A6 => "slmp_end_code_d2a6",
        0xD2A7 => "slmp_end_code_d2a7",
        0xD2A8 => "slmp_end_code_d2a8",
        0xD2A9 => "slmp_end_code_d2a9",
        0xD2AA => "slmp_end_code_d2aa",
        0xD2AB => "slmp_end_code_d2ab",
        0xD2AC => "slmp_end_code_d2ac",
        0xD2AD => "slmp_end_code_d2ad",
        0xD2AE => "slmp_end_code_d2ae",
        0xD2AF => "slmp_end_code_d2af",
        0xD2B0 => "slmp_end_code_d2b0",
        0xD2B1 => "slmp_end_code_d2b1",
        0xD2E0 => "slmp_end_code_d2e0",
        0xD2E1 => "slmp_end_code_d2e1",
        0xD602 => "slmp_end_code_d602",
        0xD605 => "slmp_end_code_d605",
        0xD611 => "slmp_end_code_d611",
        0xD612 => "slmp_end_code_d612",
        0xD613 => "slmp_end_code_d613",
        0xD614 => "slmp_end_code_d614",
        0xD615 => "slmp_end_code_d615",
        0xD616 => "slmp_end_code_d616",
        0xD617 => "slmp_end_code_d617",
        0xD618 => "slmp_end_code_d618",
        0xD619 => "slmp_end_code_d619",
        0xD61A => "slmp_end_code_d61a",
        0xD61B => "slmp_end_code_d61b",
        0xD61C => "slmp_end_code_d61c",
        0xD61D => "slmp_end_code_d61d",
        0xD61E => "slmp_end_code_d61e",
        0xD61F => "slmp_end_code_d61f",
        0xD620 => "slmp_end_code_d620",
        0xD621 => "slmp_end_code_d621",
        0xD622 => "slmp_end_code_d622",
        0xD623 => "slmp_end_code_d623",
        0xD624 => "slmp_end_code_d624",
        0xD625 => "slmp_end_code_d625",
        0xD626 => "slmp_end_code_d626",
        0xD628 => "slmp_end_code_d628",
        0xD629 => "slmp_end_code_d629",
        0xD62A => "slmp_end_code_d62a",
        0xD62B => "slmp_end_code_d62b",
        0xD630 => "slmp_end_code_d630",
        0xD634 => "slmp_end_code_d634",
        0xD635 => "slmp_end_code_d635",
        0xD636 => "slmp_end_code_d636",
        0xD637 => "slmp_end_code_d637",
        0xD638 => "slmp_end_code_d638",
        0xD639 => "slmp_end_code_d639",
        0xD63D => "slmp_end_code_d63d",
        0xD63E => "slmp_end_code_d63e",
        0xD641 => "slmp_end_code_d641",
        0xD701 => "slmp_end_code_d701",
        0xD706 => "slmp_end_code_d706",
        0xD70B => "slmp_end_code_d70b",
        0xD720 => "slmp_end_code_d720",
        0xD721 => "slmp_end_code_d721",
        0xD722 => "slmp_end_code_d722",
        0xD723 => "slmp_end_code_d723",
        0xD724 => "slmp_end_code_d724",
        0xD725 => "slmp_end_code_d725",
        0xD726 => "slmp_end_code_d726",
        0xD727 => "slmp_end_code_d727",
        0xD728 => "slmp_end_code_d728",
        0xD729 => "slmp_end_code_d729",
        0xD731 => "slmp_end_code_d731",
        0xD740 => "slmp_end_code_d740",
        0xD741 => "slmp_end_code_d741",
        0xD742 => "slmp_end_code_d742",
        0xD743 => "slmp_end_code_d743",
        0xD744 => "slmp_end_code_d744",
        0xD783 => "slmp_end_code_d783",
        0xD784 => "slmp_end_code_d784",
        0xD806 => "slmp_end_code_d806",
        0xD840 => "slmp_end_code_d840",
        0xD841 => "slmp_end_code_d841",
        0xD842 => "slmp_end_code_d842",
        0xD843 => "slmp_end_code_d843",
        0xD844 => "slmp_end_code_d844",
        0xD902 => "slmp_end_code_d902",
        0xD903 => "slmp_end_code_d903",
        0xD905 => "slmp_end_code_d905",
        0xD906 => "slmp_end_code_d906",
        0xD909 => "slmp_end_code_d909",
        0xD90A => "slmp_end_code_d90a",
        0xD90B => "slmp_end_code_d90b",
        0xD90C => "slmp_end_code_d90c",
        0xD90D => "slmp_end_code_d90d",
        0xD90E => "slmp_end_code_d90e",
        0xD90F => "slmp_end_code_d90f",
        0xD910 => "slmp_end_code_d910",
        0xD911 => "slmp_end_code_d911",
        0xD912 => "slmp_end_code_d912",
        0xD913 => "slmp_end_code_d913",
        0xD914 => "slmp_end_code_d914",
        0xD915 => "slmp_end_code_d915",
        0xD916 => "slmp_end_code_d916",
        0xD917 => "slmp_end_code_d917",
        0xD918 => "slmp_end_code_d918",
        0xDA00 => "slmp_end_code_da00",
        0xDA01 => "slmp_end_code_da01",
        0xDA10 => "slmp_end_code_da10",
        0xDA11 => "slmp_end_code_da11",
        0xDA12 => "slmp_end_code_da12",
        0xDA13 => "slmp_end_code_da13",
        0xDA14 => "slmp_end_code_da14",
        0xDA15 => "slmp_end_code_da15",
        0xDA16 => "slmp_end_code_da16",
        0xDA17 => "slmp_end_code_da17",
        0xDA19 => "slmp_end_code_da19",
        0xE006 => "slmp_end_code_e006",
        0xE102 => "slmp_end_code_e102",
        0xE103 => "slmp_end_code_e103",
        0xE120 => "slmp_end_code_e120",
        0xE121 => "slmp_end_code_e121",
        0xE122 => "slmp_end_code_e122",
        0xE123 => "slmp_end_code_e123",
        0xE160 => "slmp_end_code_e160",
        0xE162 => "slmp_end_code_e162",
        0xE163 => "slmp_end_code_e163",
        0xE164 => "slmp_end_code_e164",
        0xE165 => "slmp_end_code_e165",
        0xE166 => "slmp_end_code_e166",
        0xE170 => "slmp_end_code_e170",
        0xE171 => "slmp_end_code_e171",
        0xE172 => "slmp_end_code_e172",
        0xE173 => "slmp_end_code_e173",
        0xE174 => "slmp_end_code_e174",
        0xE175 => "slmp_end_code_e175",
        0xE176 => "slmp_end_code_e176",
        0xE177 => "slmp_end_code_e177",
        0xE178 => "slmp_end_code_e178",
        0xE179 => "slmp_end_code_e179",
        0xE17A => "slmp_end_code_e17a",
        0xE17B => "slmp_end_code_e17b",
        0xE17C => "slmp_end_code_e17c",
        0xE17D => "slmp_end_code_e17d",
        0xE17E => "slmp_end_code_e17e",
        0xE17F => "slmp_end_code_e17f",
        0xE180 => "slmp_end_code_e180",
        0xE181 => "slmp_end_code_e181",
        0xE182 => "slmp_end_code_e182",
        0xE183 => "slmp_end_code_e183",
        0xE184 => "slmp_end_code_e184",
        0xE185 => "slmp_end_code_e185",
        0xE186 => "slmp_end_code_e186",
        0xE201 => "slmp_end_code_e201",
        0xE203 => "slmp_end_code_e203",
        0xE204 => "slmp_end_code_e204",
        0xE205 => "slmp_end_code_e205",
        0xE206 => "slmp_end_code_e206",
        0xE207 => "slmp_end_code_e207",
        0xE208 => "slmp_end_code_e208",
        0xE20A => "slmp_end_code_e20a",
        0xE20B => "slmp_end_code_e20b",
        0xE20F => "slmp_end_code_e20f",
        0xE211 => "slmp_end_code_e211",
        0xE212 => "slmp_end_code_e212",
        0xE213 => "slmp_end_code_e213",
        0xE215 => "slmp_end_code_e215",
        0xE216 => "slmp_end_code_e216",
        0xE218 => "slmp_end_code_e218",
        0xE21B => "slmp_end_code_e21b",
        0xE21C => "slmp_end_code_e21c",
        0xE21E => "slmp_end_code_e21e",
        0xE21F => "slmp_end_code_e21f",
        0xE221 => "slmp_end_code_e221",
        0xE222 => "slmp_end_code_e222",
        0xE223 => "slmp_end_code_e223",
        0xE224 => "slmp_end_code_e224",
        0xE225 => "slmp_end_code_e225",
        0xE226 => "slmp_end_code_e226",
        0xE228 => "slmp_end_code_e228",
        0xE229 => "slmp_end_code_e229",
        0xE22A => "slmp_end_code_e22a",
        0xE22B => "slmp_end_code_e22b",
        0xE22C => "slmp_end_code_e22c",
        0xE22D => "slmp_end_code_e22d",
        0xE236 => "slmp_end_code_e236",
        0xE237 => "slmp_end_code_e237",
        0xE241 => "slmp_end_code_e241",
        0xE242 => "slmp_end_code_e242",
        0xE243 => "slmp_end_code_e243",
        0xE244 => "slmp_end_code_e244",
        0xE245 => "slmp_end_code_e245",
        0xE24F => "slmp_end_code_e24f",
        0xE251 => "slmp_end_code_e251",
        0xE254 => "slmp_end_code_e254",
        0xE255 => "slmp_end_code_e255",
        0xE256 => "slmp_end_code_e256",
        0xE257 => "slmp_end_code_e257",
        0xE258 => "slmp_end_code_e258",
        0xE259 => "slmp_end_code_e259",
        0xE25A => "slmp_end_code_e25a",
        0xE25B => "slmp_end_code_e25b",
        0xE262 => "slmp_end_code_e262",
        0xE264 => "slmp_end_code_e264",
        0xE265 => "slmp_end_code_e265",
        0xE266 => "slmp_end_code_e266",
        0xE267 => "slmp_end_code_e267",
        0xE268 => "slmp_end_code_e268",
        0xE269 => "slmp_end_code_e269",
        0xE26A => "slmp_end_code_e26a",
        0xE26C => "slmp_end_code_e26c",
        0xE26D => "slmp_end_code_e26d",
        0xE26E => "slmp_end_code_e26e",
        0xE26F => "slmp_end_code_e26f",
        0xE271 => "slmp_end_code_e271",
        0xE272 => "slmp_end_code_e272",
        0xE273 => "slmp_end_code_e273",
        0xE274 => "slmp_end_code_e274",
        0xE277 => "slmp_end_code_e277",
        0xE278 => "slmp_end_code_e278",
        0xE279 => "slmp_end_code_e279",
        0xE27A => "slmp_end_code_e27a",
        0xE27B => "slmp_end_code_e27b",
        0xE27C => "slmp_end_code_e27c",
        0xE27D => "slmp_end_code_e27d",
        0xE286 => "slmp_end_code_e286",
        0xE2A0 => "slmp_end_code_e2a0",
        0xE2A1 => "slmp_end_code_e2a1",
        0xE2A2 => "slmp_end_code_e2a2",
        0xE2A3 => "slmp_end_code_e2a3",
        0xE2A4 => "slmp_end_code_e2a4",
        0xE2A5 => "slmp_end_code_e2a5",
        0xE2A6 => "slmp_end_code_e2a6",
        0xE2A7 => "slmp_end_code_e2a7",
        0xE2A8 => "slmp_end_code_e2a8",
        0xE2A9 => "slmp_end_code_e2a9",
        0xE2AA => "slmp_end_code_e2aa",
        0xE2AB => "slmp_end_code_e2ab",
        0xE2AC => "slmp_end_code_e2ac",
        0xE2AD => "slmp_end_code_e2ad",
        0xE2AE => "slmp_end_code_e2ae",
        0xE2AF => "slmp_end_code_e2af",
        0xE2B0 => "slmp_end_code_e2b0",
        0xE501 => "slmp_end_code_e501",
        0xE502 => "slmp_end_code_e502",
        0xE503 => "slmp_end_code_e503",
        0xE504 => "slmp_end_code_e504",
        0xE505 => "slmp_end_code_e505",
        0xE521 => "slmp_end_code_e521",
        0xE5F0 => "slmp_end_code_e5f0",
        0xE5F1 => "slmp_end_code_e5f1",
        0xE5F8 => "slmp_end_code_e5f8",
        0xE840 => "slmp_end_code_e840",
        0xE841 => "slmp_end_code_e841",
        0xE842 => "slmp_end_code_e842",
        0xE843 => "slmp_end_code_e843",
        0xE844 => "slmp_end_code_e844",
        0xEA00 => "slmp_end_code_ea00",
        0xEA01 => "slmp_end_code_ea01",
        _ => "unknown_plc_end_code",
    }
}

/// Return the English error detail/cause message for an SLMP end code.
pub fn end_code_message_en(end_code: u16) -> Option<&'static str> {
    match end_code {
        0x1080 => Some("The number of writes to the flash ROM has exceeded 100000."),
        0x1120 => Some(
            "Clock setting has failed when the system is powered on or the CPU module is reset.",
        ),
        0x1124 => Some(
            "・The default gateway is not set correctly.・The gateway IP address is not set correctly.・The default gateway/gateway IP address (network address after the subnet mask) is different from that of the IP address of the own node.",
        ),
        0x1128 => Some("The port number is incorrect."),
        0x1129 => Some("The port number of the external device is not set correctly."),
        0x112C => Some("The request using all stations specification has failed."),
        0x112D => Some(
            "The data was sent to the external device while the IP address setting of the device set in \"External Device Configuration\" under \"Basic Settings\" was incorrect.",
        ),
        0x112E => Some("A connection could not be established in the open processing."),
        0x1133 => Some(
            "The response send failed during socket communications or communications using the fixed buffer.",
        ),
        0x1134 => Some(
            "A TCP ULP timeout error has occurred in the TCP/IP communication. (The external device does not send an ACK response.)",
        ),
        0x1152 => Some(
            "・The IP address is not set correctly.・The same IP address has been set to port 1 and port 2 of the Ethernet-equipped module.",
        ),
        0x1155 => Some(
            "・The specified connection was already closed in TCP/IP communications.・Open processing is not performed.",
        ),
        0x1157 => Some(
            "・The specified connection was already closed in UDP/IP communications.・Open processing is not performed.",
        ),
        0x1158 => Some(
            "・The receive buffer or send buffer is not sufficient.・The window size of the external device is not sufficient.",
        ),
        0x1165 => Some("Data was not sent correctly with UDP/IP."),
        0x1166 => Some("Data was not sent correctly with TCP/IP."),
        0x1167 => Some("Unsent data found, but could not be sent."),
        0x1180 => Some(
            "・The same IP address has been set as the system A IP address, system B IP address, and/or control system IP address.・Network addresses of the system A IP address, system B IP address, and control system IP address are different.",
        ),
        0x1800 => Some("A connection failure was detected in the network."),
        0x1801 => Some("IP address of the external device could not be acquired."),
        0x1811 => Some("An error was detected in the CPU module."),
        0x1830 => Some(
            "Number of reception requests of transient transmission (link dedicated instruction) exceeded upper limit of simultaneously processable requests.",
        ),
        0x1845 => Some(
            "Too many processings of transient transmission (link dedicated instruction) and cannot perform transient transmission.",
        ),
        0x1860 => Some(
            "Baton pass stops with an error of communication line or CC-Link IE Controller Network-equipped module.",
        ),
        0x1D01 => Some(
            "・\"Network Synchronous Communication\" in \"Network Configuration Settings\" under \"Basic Settings\" of the master station does not match the network synchronization communication setting of the controlled device station (synchronization enable/disabled).・The device station where the \"Network Synchronous Communication\" in \"Network Configuration Settings\" under \"Basic Settings\" of the master station is set to \"Synchronous\" does not support \"Network Synchronous Communication\".",
        ),
        0x1D10 => Some("Cyclic transmission skip occurred."),
        0x1D20 => Some(
            "The module cannot normally communicate with the synchronized device station on CC-Link IE Field Network.",
        ),
        0x1F07 => Some(
            "An error occurs in the protocol data set in \"Simple Device Communication Setting\".",
        ),
        0x20E0 => Some("The module cannot communicate with the CPU module."),
        0x2160 => Some("Overlapping IP addresses were detected."),
        0x2220 => Some(
            "・A network module having the firmware version not supporting the simple CPU communication function is used.・The number of simple CPU communication settings is 65 or more.・The parameter setting is corrupted.・The parameter is set for extending the link points extended setting, however the CPU module and network modules do not support the link points extended setting.",
        ),
        0x2221 => Some(
            "・The set value is out of the range.・The network number set on the own station is different from one on the control station・The own station is set to extended mode, however, the control station is set to normal mode. Or, the own station is set to normal mode and the control station is set to extended mode.・The own station is set to \"Extend\" of \"Link points extended setting\" in \"Application Settings\", however, the control station is set to \"Not to Extend\". Or, the own station is set to \"Not to Extend\" and the control station is set to \"Extend\".・The set value is out of the range. Or the parameter of the master station that requires the reset of the device station is changed.",
        ),
        0x2250 => Some(
            "The protocol setting data stored in the CPU module is not for the Ethernet-equipped module.",
        ),
        0x24C0 => Some("An error was detected on the system bus."),
        0x24C1 => Some("An error was detected on the system bus."),
        0x24C2 => Some("An error was detected on the system bus."),
        0x24C3 => Some("An error was detected on the system bus."),
        0x24C6 => Some("An error was detected on the system bus."),
        0x2600 => Some(
            "The cyclic processing does not finish before the start timing for the next inter-module synchronization cycle.",
        ),
        0x2610 => Some(
            "An inter-module synchronization signal error (synchronization loss) was detected.",
        ),
        0x3000 => Some(
            "・Any of following items are set in the module which is set as a target in \"Inter-module Synchronous Setting\" in the [Inter-module Synchronous Setting] tab of the \"System Parameter\" window.・\"Setting Method\" under \"Station No.\" in \"Required Settings\" is set to \"Program\".・\"Setting Method of Basic/Application Settings\" under \"Parameter Setting Method\" in \"Required Settings\" is set to \"Program\".・\"Station Type\" under \"Station Type\" in \"Required Settings\" is set to \"Submaster Station\".・\"Network Topology\" in \"Basic Settings\" is set to \"Ring\".・\"Link Scan Mode\" under \"Supplementary Cyclic Settings\" in \"Application Settings\" is set to \"Constant Link Scan\" or \"Sequence Scan Synchronous Setting\".・A station in which \"Station Type\" is set to \"Submaster station\" is set in \"Network Configuration Settings\" of \"Basic Settings\".・Although a device station in which \"Network Synchronous Communication\" in \"Network Configuration Settings\" of \"Basic Settings\" is set to \"Synchronous\" exists, the system parameter and control CPU are in the any of following states.・The master/local module is not set as the target module in \"Inter-module Synchronous Setting\" in the [Inter-module Synchronous Setting] tab of the \"System Parameter\" window.・The control CPU is a CPU module in which the inter-module synchronization function cannot be used.",
        ),
        0x3001 => Some(
            "・A station with the same station number was found in the same network.・Multiple control stations were detected in the same network.・A station with the same station number was found in the same network.・Multiple master stations and submaster stations were detected in the same network.・A station of CC-Link IE Controller Network (Ethernet cable) was found in the same network.",
        ),
        0x3004 => Some(
            "The number of points set in \"RWw/RWr Setting\" in \"Network Configuration Settings\" of \"Basic Settings\" for the safety station is less than the number of points used in the system. The number of points used in the system: The number of points used in the safety communications with the version set in \"Safety Protocol Version\" in \"Safety Communication Setting\" under \"Application Settings\".",
        ),
        0x3005 => Some("Parameters for the master station and submaster station do not match."),
        0x3006 => Some("Pairing is not set to the stations in a redundant system."),
        0x3007 => Some("Pairing is set to the stations not included in a redundant system."),
        0x3008 => Some(
            "・In the redundant system with redundant extension base unit, a module name other than \"RJ71EN71(E+E)\" and \"RJ71EN71(Q)\" is set on the extension base.・\"RJ71GP21-SX\" or \"RJ71GP21S-SX\" is selected for the module name in a redundant system.・\"RJ71GP21-SX (R)\" or \"RJ71GP21S-SX (R)\" is selected for the module name in a system other than a redundant system.・\"RJ71GF11-T2\" is selected for the module name in a redundant system.・\"RJ71GF11-T2 (MR)\", \"RJ71GF11-T2 (SR)\", or \"RJ71GF11-T2 (LR)\" is selected for the module name in a system other than a redundant system.",
        ),
        0x3019 => Some(
            "・When mounting a module on the main base unit in the redundant system, \"Not Use\" is set to \"To Use or Not to Use Redundant System Settings\" under \"Application Settings\".・When mounting a module on the extension base unit in the redundant system with redundant extension base unit, \"Use\" is set to \"To Use or Not to Use Redundant System Settings\" under \"Application Settings\".",
        ),
        0x301A => Some(
            "When mounting a module on the extension base unit in the redundant system with redundant extension base unit, \"Enable\" is set to \"Dynamic Routing\" under \"Application Settings\".",
        ),
        0x301B => Some(
            "When mounting a module on the extension base unit in the redundant system with redundant extension base unit, \"Use\" is set to \"IP Packet Transfer Function\" under \"Application Settings\".",
        ),
        0x301C => Some(
            "・Incorrect protocol data is set in \"Simple Device Communication Setting\".・A packet used is not supported by the current firmware version.",
        ),
        0x301D => Some("A function used is not supported by the current firmware version."),
        0x301E => Some(
            "The \"Resource Setting\" is incorrectly set in \"Simple Device Communication Setting\" of the port 1 or port 2.",
        ),
        0x301F => Some(
            "The safety protocol version set in \"Safety Communication Setting\" is not supported by the network module or CPU module (Safety CPU or SIL2 Process CPU).",
        ),
        0x3020 => Some("A value of the port number is out of range."),
        0x3022 => Some(
            "The total number of external devices and communication destinations set using parameters exceeds 64.",
        ),
        0x3023 => Some(
            "\"Port No. Host Station\" set in \"Simple Device Communication Setting\" overlaps a value set for another function.",
        ),
        0x3026 => Some(
            "The module does not support \"Extension 2\", which is set in \"Resource Setting\" under \"Simple Device Communication Setting\".",
        ),
        0x3040 => Some("Response data of the dedicated instruction cannot be created."),
        0x3060 => Some("The send/receive data size exceeds the allowable range."),
        0x31D0 => Some(
            "・In \"Safety Communication Setting\" under \"Application Settings\", safety protocol version 2 is selected for a communication destination that does not support safety protocol version 2.・The number of safety connections with the same communication destination that is set with safety protocol version 2 is set to 17 or more.",
        ),
        0x3600 => Some(
            "The inter-module synchronization cycle setting does not match the master station setting.",
        ),
        0x3601 => Some(
            "\"Network Synchronous Communication\" in \"Network Configuration Settings\" under \"Basic Settings\" of the master station does not match the inter-module synchronization target module of the own station.",
        ),
        0x3602 => Some("Inter-module synchronization cycle failure occurred between networks."),
        0x3C00 => Some(
            "・A hardware failure has been detected.・In a redundant system configuration, the control system was powered off or reset without the tracking cable connected.",
        ),
        0x3C01 => Some(
            "・A hardware failure has been detected.・In a redundant system configuration, the control system was powered off or reset without the tracking cable connected.",
        ),
        0x3C02 => Some(
            "・A hardware failure has been detected.・In a redundant system configuration, the control system was powered off or reset without the tracking cable connected.",
        ),
        0x3C03 => Some(
            "・A hardware failure has been detected.・In a redundant system configuration, the control system was powered off or reset without the tracking cable connected.",
        ),
        0x3C0F => Some("A hardware failure has been detected."),
        0x3C10 => Some(
            "・A hardware failure has been detected.・A function which is not supported was used. (When Ethernet cables are used)",
        ),
        0x3C11 => Some("A hardware failure has been detected."),
        0x3C13 => Some("A hardware failure has been detected."),
        0x3C14 => Some("A hardware failure has been detected."),
        0x3C2F => Some("An error was detected in the memory."),
        0x3E00 => Some("An error was detected in the network module."),
        0x3E01 => Some("Network type of the own station is unexpected setting."),
        0xC001 => Some(
            "・The IP address setting value of the E71 for the initial processing is incorrect.・The setting value of the subnet mask field for the router relay function is incorrect.",
        ),
        0xC012 => Some("The port number used in a connection already opened is set. (For TCP/IP)"),
        0xC013 => Some("The port number used in a connection already opened is set. (For UDP/IP)"),
        0xC015 => Some(
            "・The specified IP address of the external device for the open processing is incorrect.・The specified IP address of the external device of the dedicated instruction is incorrect.",
        ),
        0xC016 => Some(
            "The open processing of the connection specified for pairing open has been already completed.",
        ),
        0xC018 => Some("The specified IP address of the external device is incorrect."),
        0xC020 => Some("The send/receive data length exceeds the allowable range."),
        0xC021 => Some(
            "An abnormal end response was received for communications using the fixed buffer and random access buffer.",
        ),
        0xC022 => Some(
            "・A response could not be received within the response monitoring timer value.・The connection with the external device was closed while waiting for a response.",
        ),
        0xC024 => Some(
            "・Communications using the fixed buffer or communications using a random access buffer were executed when communication method is set to the \"Predefined Protocol\" connection.・Predefined protocol was executed when communication method is set to \"Fixed Buffer (Procedure Exist)\" or \"Fixed Buffer (No Procedure)\" connection.",
        ),
        0xC025 => Some(
            "There is an error in the usage setting area when starting the open processing by the CONOPEN/OPEN instruction or I/O signals.",
        ),
        0xC026 => Some(
            "An error has occurred when reading/writing/verifying the predefined protocol setting data.",
        ),
        0xC027 => Some("Message send of the socket communications has failed."),
        0xC028 => Some("Message send of the fixed buffer has failed."),
        0xC029 => Some(
            "・Description of control data is not correct.・Open instruction was executed through open settings parameter even though parameters are not set.",
        ),
        0xC035 => Some(
            "The existence of the external device could not be checked within the response monitoring timer value.",
        ),
        0xC040 => Some(
            "・Not all the data could be received within the response monitoring timer value.・Sufficient data for the data length could not be received.・The remaining part of the message divided at the TCP/IP level could not be received within the response monitoring timer value.",
        ),
        0xC050 => Some(
            "When \"ASCII\" has been selected in the communication data code setting of the Ethernet-equipped module, ASCII code data which cannot be converted into binary code data has been received.",
        ),
        0xC051 => Some(
            "・The number of read/write points from/to the device of SLMP message is out of the allowable range in the CPU module (in units of words).・The number of write points for the long counter of SLMP message is not in two-word units.",
        ),
        0xC052 => Some(
            "The number of read/write points from/to the device of SLMP message is out of the allowable range in the CPU module (in units of bits).",
        ),
        0xC053 => Some(
            "The number of read/write points from/to the random device of SLMP message is out of the allowable range in the CPU module (in units of bits).",
        ),
        0xC054 => Some(
            "The number of read/write points from/to the random device of SLMP message is out of the allowable range in the CPU module (in units of words, double words).",
        ),
        0xC055 => Some(
            "The read/write size from/to the file data of SLMP message is out of the allowable range.",
        ),
        0xC056 => Some("The read/write request exceeds the largest address."),
        0xC057 => Some(
            "The request data length of the SLMP message does not match with the number of data in the character (a part of text).",
        ),
        0xC058 => Some(
            "The request data length of the SLMP message after the ASCII/binary conversion does not match with the number of data in the character (a part of text).",
        ),
        0xC059 => Some(
            "・The specified command and subcommand of the SLMP message are incorrect・The function which is not supported by the target device was executed.",
        ),
        0xC05A => Some(
            "The Ethernet-equipped module cannot read/write data from/to the device specified by the SLMP message.",
        ),
        0xC05B => Some(
            "The Ethernet-equipped module cannot read/write data from/to the device specified by the SLMP message.",
        ),
        0xC05C => Some(
            "・The received request data of the SLMP message is incorrect.・The setting value of the communication setting when the iQSS function is executed is out of range.・When the iQSS function is executed, the items of communication setting which cannot be set on the target device are set.・When the iQSS function is executed, the required setting items have not been set to the target device.",
        ),
        0xC05D => Some(
            "The \"Monitor Request\" command is received before the monitor registration is performed by \"Monitor Registration/Clear\" command of the SLMP message.",
        ),
        0xC05E => Some(
            "・The time between received the SLMP message from the Ethernet-equipped module and returned response from the access destination exceeded the monitoring timer value set in the SLMP command.・An SLMP request message to which a command without a response message is specified is send to a module with the other network number as an access destination.",
        ),
        0xC05F => Some(
            "This request cannot be executed to the access destination specified by the SLMP message.",
        ),
        0xC060 => Some("The request details for bit devices of the SLMP message is incorrect."),
        0xC061 => Some(
            "・The request data length of the SLMP message does not match with the number of data in the character (a part of text).・The write data length specified by the label write command is not even byte.・When the iQSS function is executed, incorrect frame is received.",
        ),
        0xC070 => Some(
            "The device memory cannot be extended for the access destination specified by the SLMP message.",
        ),
        0xC071 => Some(
            "The number of device points for data read/write set for modules other than an R/Q/QnACPU is out of the range.",
        ),
        0xC072 => Some(
            "The request details of the SLMP message is incorrect. (For example, a request for data read/write in bit units has been issued to a word device.)",
        ),
        0xC073 => Some(
            "The access destination of the SLMP message cannot issue this request. (For example, the number of double word access points cannot be specified for modules other than an R/Q/QnACPU.)",
        ),
        0xC075 => Some("The request data length for the label access is out of range."),
        0xC081 => Some(
            "The termination processing for the Ethernet-equipped module that is involved with the reinitialization processing is being performed, and arrival of link dedicated instructions cannot be checked.",
        ),
        0xC083 => Some(
            "The communication processing was abnormally ended in the link dedicated instruction communications",
        ),
        0xC084 => Some(
            "The communication processing was abnormally ended in the link dedicated instruction communications",
        ),
        0xC085 => Some(
            "The target station's channel specified by the link dedicated instruction SEND is currently in use.",
        ),
        0xC0B2 => Some(
            "There is no sufficient space in the receive buffer or the send buffer of the relay station or external station for the MELSOFT connection, link dedicated instructions, or SLMP. (Send · receive buffer full error)",
        ),
        0xC0B3 => Some("A request that cannot be processed was issued from the CPU module."),
        0xC0B6 => Some("The channel specified by the dedicated instruction is out of the range."),
        0xC0BA => Some(
            "Since the close processing is in execution using the CONCLOSE/CLOSE instruction, a send request cannot be accepted.",
        ),
        0xC0C4 => Some("The UINI instruction has been executed during communications."),
        0xC0D0 => Some("The specified data length of the link dedicated instruction is incorrect."),
        0xC0D1 => Some("The number of resends of the link dedicated instruction is incorrect."),
        0xC0D3 => Some(
            "The number of relay stations to communicate with other networks exceeds the allowable range.",
        ),
        0xC0D4 => Some(
            "The number of relay stations to communicate with other networks exceeds the allowable range.",
        ),
        0xC0D5 => Some("The number of retries of the link dedicated instruction is incorrect."),
        0xC0D6 => Some(
            "The network number or station number of the link dedicated instruction is incorrect.",
        ),
        0xC0D7 => Some("Data were sent without the initial processing completed."),
        0xC0D8 => Some("The number of specified blocks exceeded the range."),
        0xC0D9 => Some("The specified subcommand of the SLMP message is incorrect."),
        0xC0DA => Some(
            "A response to the PING test could not be received within the time of the communication time check.",
        ),
        0xC0DB => Some(
            "The IP address and host name of the target module where the PING test is execute are incorrect.",
        ),
        0xC0DE => Some("Data could not be received within the specified arrival monitoring time."),
        0xC101 => Some("A response could not be received from the DNS server."),
        0xC1A2 => Some(
            "・A response to the request could not be received.・In transient transmission, the number of relay to other networks exceeded seven.",
        ),
        0xC1A4 => Some(
            "・Any of the specified command, subcommand, or request destination module I/O number of the SLMP message is incorrect.・The specified clear function set by the ERRCLEAR instruction is incorrect.・The specified information to be read set by the ERRRD instruction is incorrect.・The Ethernet diagnostics, CC-Link IE Field Network diagnostics, or CC-Link IE Controller Network diagnostics was tried to be used when the engineering tool is directly connected to the Ethernet port of the RJ71EN71.・The function which is not supported by the target device was executed.",
        ),
        0xC1A5 => Some("The specified target station or clear target is incorrect."),
        0xC1A6 => Some("The specified connection number is incorrect."),
        0xC1A7 => Some("The specified network number is incorrect."),
        0xC1A8 => Some("The specified station number is incorrect."),
        0xC1A9 => Some("The specified device number is incorrect."),
        0xC1AA => Some("The specified device name is incorrect."),
        0xC1AC => Some("The specified number of resends is incorrect."),
        0xC1AD => Some("The specified data length is incorrect."),
        0xC1AF => Some("The specified port number is incorrect."),
        0xC1B0 => {
            Some("The open processing of the specified connection has been already completed.")
        }
        0xC1B1 => Some("The open processing of the specified connection has not been completed."),
        0xC1B2 => Some(
            "The open or close processing using CONOPEN/CONCLOSE/OPEN/CLOSE instruction is being executed in the specified connection.",
        ),
        0xC1B3 => {
            Some("Another send or receive instruction is being executed in the specified channel.")
        }
        0xC1B4 => Some("The specified arrival monitoring time is incorrect."),
        0xC1B8 => {
            Some("The RECV instruction was executed for the channel that had not received data.")
        }
        0xC1B9 => {
            Some("The CONOPEN/OPEN instruction cannot be executed for the specified connection.")
        }
        0xC1BA => {
            Some("The dedicated instruction was executed with the initialization not completed.")
        }
        0xC1BB => {
            Some("The target station CPU type of the link dedicated instruction is incorrect.")
        }
        0xC1BC => Some("The target network number of the link dedicated instruction is incorrect."),
        0xC1BD => Some("The target station number of the link dedicated instruction is incorrect."),
        0xC1BE => Some("The command code of the dedicated instruction is incorrect."),
        0xC1BF => Some("The channel used in the dedicated instruction is incorrect."),
        0xC1C0 => Some("The transient data is incorrect."),
        0xC1C1 => Some("The transient data is incorrect."),
        0xC1C2 => Some("When the dedicated instruction was executed, data was received twice."),
        0xC1C4 => {
            Some("The arrival check of the link dedicated instruction was completed with an error.")
        }
        0xC1C5 => {
            Some("A dedicated instruction which the target station does not support was executed.")
        }
        0xC1C6 => Some(
            "The execution or error completion type of the dedicated instruction is incorrect.",
        ),
        0xC1C7 => Some("The request type of the REQ instruction is incorrect."),
        0xC1C8 => Some("The channel specified in the dedicated instruction is in use."),
        0xC1C9 => Some("The device specification for the ZNRD/ZNWR instruction is not correct."),
        0xC1CA => Some("The device specification for the ZNRD/ZNWR instruction is not correct."),
        0xC1CB => Some("The transient data is incorrect."),
        0xC1CC => Some(
            "A response of the data length that exceeds the allowable range was received by the SLMPSND instruction.",
        ),
        0xC1CD => Some("Message send of the SLMPSND instruction has failed."),
        0xC1D0 => Some("The requested module I/O No. of the dedicated instruction is incorrect."),
        0xC1D2 => {
            Some("The target station IP address of the link dedicated instruction is incorrect.")
        }
        0xC1D3 => Some(
            "The dedicated instruction not supported by the communication method of the connection was executed.",
        ),
        0xC200 => Some("The remote password is incorrect."),
        0xC201 => Some(
            "The remote password status of the port used for communications is in the lock status.",
        ),
        0xC202 => {
            Some("When another station was accessed, the remote password could not be unlocked.")
        }
        0xC203 => Some("An error has occurred by checking the remote password."),
        0xC204 => Some(
            "The device is different from the one requesting the remote password unlock processing.",
        ),
        0xC205 => {
            Some("When another station was accessed, the remote password could not be unlocked.")
        }
        0xC207 => Some("The file name has too many characters."),
        0xC208 => Some("The password length is out of range."),
        0xC400 => Some(
            "The ECPRTCL instruction was executed when Predefined protocol ready is not completed.",
        ),
        0xC401 => Some(
            "The protocol number specified by the ECPRTCL instruction is not registered in the Ethernet-equipped module.",
        ),
        0xC402 => Some(
            "An error has occurred in the protocol setting data registered in the Ethernet-equipped module and the ECPRTCL instruction cannot be executed.",
        ),
        0xC403 => Some("Multiple dedicated instructions was executed simultaneously."),
        0xC404 => Some("The protocol being executed by the ECPRTCL instruction was canceled."),
        0xC405 => Some("The protocol number specified by the ECPRTCL instruction is incorrect."),
        0xC406 => {
            Some("The continuous protocol execution count of the ECPRTCL instruction is incorrect.")
        }
        0xC407 => Some("The connection number specified by the ECPRTCL instruction is incorrect."),
        0xC408 => Some(
            "An error has occurred when the send processing of the predefined protocol using the ECPRTCL instruction was performed.",
        ),
        0xC410 => Some("Receive waiting time of the ECPRTCL instruction timed out."),
        0xC412 => {
            Some("The data which cannot be converted from ASCII to binary code was received.")
        }
        0xC413 => Some(
            "The number of digits of the received data using the predefined protocol is not sufficient.",
        ),
        0xC414 => Some(
            "The number of digits of the received data using the predefined protocol is incorrect.",
        ),
        0xC417 => Some(
            "The data length or data quantity of the received data using the predefined protocol is out of range.",
        ),
        0xC420 => Some("Protocol setting data write has failed."),
        0xC421 => Some(
            "Writing was requested to the module whose flash ROM write count had exceeded the limit.",
        ),
        0xC430 => {
            Some("Protocol setting data was written during the ECPRTCL instruction execution.")
        }
        0xC431 => Some(
            "Close processing of the connection was performed during the ECPRTCL instruction execution.",
        ),
        0xC440 => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC441 => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC442 => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC443 => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC444 => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC445 => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC446 => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC447 => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC448 => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC449 => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC44A => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC44B => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC44C => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC44D => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC44E => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC44F => Some(
            "A communication error has occurred with an engineering tool when executing the Ethernet diagnostics.",
        ),
        0xC610 => Some("The module processing was completed with an error."),
        0xC611 => Some("The module processing was completed with an error."),
        0xC612 => Some("The module processing was completed with an error."),
        0xC613 => Some("The module processing was completed with an error."),
        0xC614 => Some("The module processing was completed with an error."),
        0xC615 => Some("The module processing was completed with an error."),
        0xC616 => Some("Connection of the control port to the FTP server failed."),
        0xC617 => Some("Disconnection of the control port to the FTP server failed."),
        0xC618 => Some("Login to the FTP server failed."),
        0xC619 => Some("Execution of the FTP command to the FTP server failed."),
        0xC620 => Some("Connection of the data transfer port to the FTP server failed."),
        0xC621 => Some("Disconnection of the data transfer port to the FTP server failed."),
        0xC622 => Some("An error has occurred during file transfer to the FTP server."),
        0xC623 => Some("A response could not be received from the FTP server."),
        0xC700 => Some("The module processing was completed with an error."),
        0xC701 => Some(
            "The IP address (network number) setting is incorrect in communications using the IP packet transfer function.",
        ),
        0xC702 => Some(
            "The IP address (station number) setting is incorrect in communications using the IP packet transfer function.",
        ),
        0xC703 => Some(
            "The destination IP address (upper level) setting is incorrect in communications using the IP packet transfer function.",
        ),
        0xC704 => Some(
            "The destination IP address (lower level) setting is incorrect in communications using the IP packet transfer function.",
        ),
        0xC705 => Some("The module processing was completed with an error."),
        0xC706 => Some("The module processing was completed with an error."),
        0xC707 => Some("The module processing was completed with an error."),
        0xC708 => Some(
            "When communicating with the IP packet transfer function, \"IP Packet Transfer Function\" is set as \"Not Use\" in \"IP Packet Transfer Setting\" under \"Application Settings\" of the Ethernet-equipped module connected with the Ethernet devices.",
        ),
        0xC709 => Some("A communication error has occurred with MELSOFT direct connection."),
        0xC810 => Some(
            "Remote password authentication has failed when required. Set a correct password and retry.",
        ),
        0xC811 => Some(
            "Remote password authentication has failed when required. Set a correct password and retry after 1 minute.",
        ),
        0xC812 => Some(
            "Remote password authentication has failed when required. Set a correct password and retry after 5 minutes.",
        ),
        0xC813 => Some(
            "Remote password authentication has failed when required. Set a correct password and retry after 15 minutes.",
        ),
        0xC814 => Some(
            "Remote password authentication has failed when required. Set a correct password and retry after 60 minutes.",
        ),
        0xC815 => Some(
            "Remote password authentication has failed when required. Set a correct password and retry after 60 minutes.",
        ),
        0xC816 => Some(
            "The security function was activated and remote password authentication cannot be performed.",
        ),
        0xC840 => Some(
            "Number of transient request exceeded the upper limit of simultaneously processable requests.",
        ),
        0xC842 => {
            Some("The routing setting is not set to reach to the destination network number.")
        }
        0xC843 => Some(
            "Link dedicated instruction that cannot be executed on the network type were executed.",
        ),
        0xC844 => Some(
            "Incorrect frame is received.・Unsupported pre-conversion protocol・Unsupported frame type・Application header variable part・Application header HDS・Application header RTP・Read command not requiring response",
        ),
        0xC860 => Some(
            "The CPU response monitoring timer issued a timeout during MODBUS/TCP communications.",
        ),
        0xC861 => Some(
            "A request message containing an unsupported function code was received during MODBUS/TCP communications.",
        ),
        0xC862 => Some(
            "A request message containing an unsupported sub-code was received during MODBUS/TCP communications.",
        ),
        0xC863 => Some(
            "The MODBUS device assignment parameters have not been set for the MODBUS device specified in the received request message in MODBUS/TCP communications.",
        ),
        0xC864 => Some(
            "The range of the MODBUS devices specified in the received request message exceeded the setting range of the MODBUS device assignment parameters in MODBUS/TCP communications.",
        ),
        0xC865 => Some(
            "The range of the MODBUS devices specified in the received request message exceeded the upper limit for the MODBUS devices in MODBUS/TCP communications. (The upper limit of extended file register is 10000 and the upper limit of the MODBUS device is 65536.)",
        ),
        0xC866 => Some(
            "The start address and the number of access points of the MODBUS device specified in the received request message are incorrect in MODBUS/TCP communications.",
        ),
        0xC867 => Some(
            "The number of write points specified for the received request message does not match with the specified number of bytes in MODBUS/TCP communications.",
        ),
        0xC868 => Some(
            "The received write data size does not match with the specified number of bytes in MODBUS/TCP communications.",
        ),
        0xC869 => Some(
            "The value of the reference type specified in the received request message (FC20, FC21) is incorrect in MODBUS/TCP communications.",
        ),
        0xC86A => Some(
            "The content of the data part of the received request message is incorrect in MODBUS/TCP communications.The size of the received request message is smaller than the minimum size required for the function code, or larger than the maximum size required for the function code.",
        ),
        0xC86B => Some(
            "The content of MBAP header of the received request message is incorrect in MODBUS/TCP communications.",
        ),
        0xC86C => Some(
            "The number of received request messages exceeded the number that can be received simultaneously.",
        ),
        0xCEE0 => Some(
            "The devices supporting iQSS which were detected by the other peripheral device, or other iQSS functions were executed while the automatic detection of connected devices is in process.",
        ),
        0xCEE1 => Some("Incorrect frame is received."),
        0xCEE2 => Some("Incorrect frame is received."),
        0xCF10 => Some("Incorrect frame is received."),
        0xCF20 => Some(
            "・The setting value of the communication setting is out of range.・The items of communication setting which cannot be set on the target device are set.・The required setting items have not been set to the target device.",
        ),
        0xCF30 => Some("The parameter which is not supported by the target device was specified."),
        0xCF31 => Some("Incorrect frame is received."),
        0xCF70 => Some("An error occurred on the Ethernet communication path."),
        0xCF71 => Some("A timeout error has occurred."),
        0xCF80 => Some(
            "During the simple device communication, communications with the communication destination cannot be executed. Or the external device is disconnected.",
        ),
        0xCF81 => Some(
            "During the simple device communication, the communication has failed due to a communication timeout.",
        ),
        0xCF82 => Some(
            "During the simple device communication, the \"Send/Receive Data Length Storage Area\" or \"Send/Receive Data Count Storage Area\" value in \"Non-conversion Variable (variable length)\" or \"Conversion Variable (variable length)\" of \"Variable Number of Data\" set for the send packet exceeds the range that can be set.",
        ),
        0xCF83 => {
            Some("During the simple device communication, the data size of send data has been 0.")
        }
        0xCF84 => {
            Some("During the simple device communication, sending the send packet has failed.")
        }
        0xCF85 => Some("Data outside the range was set during simple device communication."),
        0xCF8A => Some(
            "During the simple device communication, the request to the CPU module has been failed.",
        ),
        0xCF8C => Some(
            "During the simple device communication, the receive data does not match with any of the receive packets set using the parameter.",
        ),
        0xCFB0 => {
            Some("Sending data failed due to a resend timeout while the simple CPU communication.")
        }
        0xCFB1 => Some(
            "Communications with the communication destination do not executed while the simple CPU communication. Or the external device is disconnected.",
        ),
        0xCFB2 => Some(
            "The same specified own station port number is already used for the simple CPU communication.",
        ),
        0xCFB3 => {
            Some("The request to the CPU module has failed while the simple CPU communication.")
        }
        0xCFB4 => Some(
            "An abnormal response was received from the communication destination while the simple CPU communication.",
        ),
        0xCFB5 => Some(
            "The frame received from the communication destination is incorrect while the simple CPU communication.",
        ),
        0xCFBD => Some(
            "The device specified as the communication destination is out of specification range for the simple CPU communication.",
        ),
        0xCFBE => Some(
            "A communication error has occurred with an engineering tool during the simple CPU communication diagnostics.",
        ),
        0xCFBF => Some("The simple CPU communication cannot be executed."),
        0xD000 => Some("An error was detected in the network module."),
        0xD038 => {
            Some("The target station specified in the IP communication test is not connected.")
        }
        0xD039 => Some(
            "There is a station that does not support the IP packet transfer function on the communication path of the IP communication test.",
        ),
        0xD03B => Some(
            "Enabling the remote device test function failed because the operating status of the CPU module is not in STOP state (except for a stop error occurrence).",
        ),
        0xD041 => Some("The number of communication stations is incorrect."),
        0xD080 => Some("An error was detected in the network module."),
        0xD081 => Some("An error was detected in the network module."),
        0xD082 => Some("An error was detected in the network module."),
        0xD083 => Some("An error was detected in the network module."),
        0xD0A0 => {
            Some("Transmission response wait timeout has occurred in transient transmission.")
        }
        0xD0A1 => {
            Some("Transmission completion wait timeout has occurred in transient transmission.")
        }
        0xD0A2 => {
            Some("Transmission processing wait timeout has occurred in transient transmission.")
        }
        0xD0A3 => Some("Send processing of the transient transmission has failed."),
        0xD0A4 => Some("Transient transmission failed."),
        0xD0A5 => Some("Transient transmission failed."),
        0xD0A6 => Some("Transient transmission failed."),
        0xD0C0 => Some(
            "Reserved station specification was performed again during processing of the specification.",
        ),
        0xD0C1 => Some(
            "Temporary reserved station cancel specification was performed again during processing of the specification.",
        ),
        0xD0C4 => Some(
            "Temporary error invalid station setting was performed again during processing of the setting.",
        ),
        0xD0C5 => Some(
            "Temporary error invalid station setting cancel specification was performed again during processing of the specification.",
        ),
        0xD0D0 => Some("Station number setting of the other stations has failed."),
        0xD200 => Some("When the transient transmission was executed, data was received twice."),
        0xD202 => Some("The send buffer is full."),
        0xD203 => Some(
            "The number of read data or write address of the transient transmission is incorrect.",
        ),
        0xD204 => Some("The network number of transient transmission is incorrect."),
        0xD205 => Some("The target station number of transient transmission is incorrect."),
        0xD206 => Some("The network number of transient transmission is incorrect."),
        0xD207 => Some(
            "・In transient transmission, the number of relay to other networks exceeded seven.・The transient transmission was performed from the standby system of the redundant system.",
        ),
        0xD208 => Some("The network number of transient transmission is incorrect."),
        0xD209 => Some("The target station number of transient transmission is incorrect."),
        0xD20A => Some("The target station number of transient transmission is incorrect."),
        0xD20B => Some(
            "When there was no master station, specified master station was specified for transient transmission.",
        ),
        0xD20C => Some(
            "When there was no master station, current master station was specified for transient transmission.",
        ),
        0xD20D => Some(
            "Transmission completion wait timeout has occurred in transient data transmission.",
        ),
        0xD20E => Some("The header information of transient transmission is incorrect."),
        0xD20F => Some(
            "In transient transmission, the command which cannot be requested to all or a group of stations was executed with all stations specification or group specification.",
        ),
        0xD210 => Some("The target station number of transient transmission is incorrect."),
        0xD211 => Some(
            "Transient transmission was performed when the station number of the own station has not been set yet.",
        ),
        0xD212 => Some("Transient transmission failed."),
        0xD213 => Some(
            "・The command of transient transmission is incorrect.・The CC-Link IE Field Network diagnostics was used for the network to which the relay receiving station belongs.",
        ),
        0xD214 => Some("The data length of transient transmission is incorrect."),
        0xD215 => Some(
            "The module operation mode is set to a mode in which transient transmission cannot be executed.",
        ),
        0xD216 => Some("The command of transient transmission is incorrect."),
        0xD217 => Some("The command of transient transmission is incorrect."),
        0xD218 => Some("The number of read/write data of transient transmission is incorrect."),
        0xD219 => Some("The attribute code of transient transmission is incorrect."),
        0xD21A => Some("The access code of transient transmission is incorrect."),
        0xD21B => Some("A transient transmission error was detected."),
        0xD21C => Some("A transient transmission error was detected."),
        0xD21D => Some("The network number of transient transmission is incorrect."),
        0xD21E => Some("The target station number of transient transmission is incorrect."),
        0xD21F => {
            Some("The target station type specification of the dedicated instruction is incorrect.")
        }
        0xD220 => Some("The master station does not exist."),
        0xD222 => Some("The command of transient transmission is incorrect."),
        0xD223 => Some("A transient transmission error was detected."),
        0xD224 => Some("A transient transmission error was detected."),
        0xD22E => Some("Station number setting is not available for the target station."),
        0xD22F => Some("Baton pass has not been performed in the target station."),
        0xD230 => Some("The target station of station number change is incorrectly specified."),
        0xD231 => Some(
            "The station number has been already set for the target station of station number change.",
        ),
        0xD232 => Some("The target station of station number change does not exist."),
        0xD233 => Some("The station number specified for station number change is incorrect."),
        0xD234 => Some("Baton pass has not been performed."),
        0xD235 => Some("A transient transmission error was detected."),
        0xD236 => Some("The TTL of the IP data is incorrect."),
        0xD237 => Some("The IP address setting is not correctly set."),
        0xD238 => Some("The send queue is full."),
        0xD239 => Some("SLMP transmission failed."),
        0xD23A => Some("The subheader in the SLMP data is incorrect."),
        0xD23B => Some("The network number in the SLMP data is incorrect."),
        0xD23C => Some("The target station number in the SLMP data is incorrect."),
        0xD23D => Some("The information of the device station failed to be acquired."),
        0xD23E => Some("The information of the device station failed to be acquired."),
        0xD240 => {
            Some("The network number specification of the dedicated instruction is incorrect.")
        }
        0xD241 => Some("The target station number of the dedicated instruction is incorrect."),
        0xD242 => Some("The command code of the dedicated instruction is incorrect."),
        0xD243 => Some("The channel specified in the dedicated instruction is incorrect."),
        0xD244 => Some("The transient data is incorrect."),
        0xD245 => Some("The target station number of the dedicated instruction is incorrect."),
        0xD246 => Some("The transient data is incorrect."),
        0xD247 => Some(
            "When the dedicated instruction was executed, response from the target station was received twice.",
        ),
        0xD249 => Some("The target station's CPU type of the dedicated instruction is incorrect."),
        0xD24A => Some("The arrival monitoring time of the dedicated instruction is incorrect."),
        0xD24B => {
            Some("The number of resends specified in the dedicated instruction is incorrect.")
        }
        0xD24C => {
            Some("The network number specification of the dedicated instruction is incorrect.")
        }
        0xD24D => Some("The channel specified in the dedicated instruction is incorrect."),
        0xD24E => Some("The target station setting in the dedicated instruction is incorrect."),
        0xD24F => Some(
            "The dedicated instruction was executed when the station number of the own station has not been set yet.",
        ),
        0xD251 => {
            Some("When the dedicated instruction was executed, arrival check error has occurred.")
        }
        0xD252 => Some(
            "Transmission completion wait timeout has occurred when the dedicated instruction was executed.",
        ),
        0xD253 => {
            Some("A response timeout has occurred when the dedicated instruction was executed.")
        }
        0xD254 => {
            Some("A dedicated instruction which the target station does not support was executed.")
        }
        0xD255 => Some("The target station number of the dedicated instruction is incorrect."),
        0xD256 => {
            Some("The execution/error completion type of the dedicated instruction is incorrect.")
        }
        0xD257 => Some("The request type of the REQ instruction is incorrect."),
        0xD258 => Some(
            "The control station does not exist when the dedicated instruction was executed to the specified control station or current control station.",
        ),
        0xD25A => Some("The dedicated instruction was executed specifying the channel in use."),
        0xD25B => Some("The dedicated instruction was executed specifying the channel in use."),
        0xD25C => {
            Some("The function version specification of the dedicated instruction is incorrect.")
        }
        0xD25D => Some("The transient data is incorrect."),
        0xD25E => {
            Some("Dedicated instructions which cannot be executed simultaneously were executed.")
        }
        0xD25F => Some(
            "The REMFR/REMTO/REMFRD/REMTOD instruction was executed from a module with a station type which cannot execute it.",
        ),
        0xD260 => Some(
            "The REMTO/REMTOD instruction was executed from the module with a station type which cannot execute it.",
        ),
        0xD261 => Some(
            "The CCPASET instruction was executed from the module with a station type which cannot execute it.",
        ),
        0xD262 => Some(
            "The total number of device stations specified in the CCPASET instruction is incorrect.",
        ),
        0xD263 => {
            Some("The constant link scan time setting of the CCPASET instruction is incorrect.")
        }
        0xD264 => Some("The station number setting of the CCPASET instruction is incorrect."),
        0xD265 => {
            Some("The station number specified for the CCPASET instruction is already in use.")
        }
        0xD266 => {
            Some("The device station setting information of the CCPASET instruction is incorrect.")
        }
        0xD267 => Some("The station type of the CCPASET instruction is incorrect."),
        0xD268 => Some(
            "The link device range assignment specified for each station in the CCPASET instruction is incorrect.",
        ),
        0xD269 => Some(
            "・The station type of the REMFR/REMTO/REMFRD/REMTOD instruction target station is not an intelligent device station/remote device station.・The station type of the SINFTYRD/SINFSTRD instruction target station is not an intelligent device station (remote head module).",
        ),
        0xD26A => Some(
            "The target station of the REMFR/REMTO/REMFRD/REMTOD/SINFTYRD/SINFSTRD instruction does not exist.",
        ),
        0xD26B => Some(
            "The network number setting of the CCPASET instruction execution station is incorrect.",
        ),
        0xD26C => Some(
            "The station type and station number of the CCPASET instruction execution station are incorrect.",
        ),
        0xD26F => Some(
            "The station number specified for submaster station in the CCPASET instruction is incorrect.",
        ),
        0xD270 => Some("Multiple submaster stations are set in the CCPASET instruction."),
        0xD271 => Some(
            "A submaster station is specified in the reserved station setting of the CCPASET instruction.",
        ),
        0xD272 => Some(
            "A submaster station is specified in the error invalid station setting of the CCPASET instruction.",
        ),
        0xD273 => Some("The request data size of transient transmission is incorrect."),
        0xD274 => Some("The routing setting is not correctly set."),
        0xD275 => Some(
            "Other dedicated instructions are in execution and the executed instruction cannot be processed.",
        ),
        0xD276 => {
            Some("The station type of the dedicated instruction target station is incorrect.")
        }
        0xD277 => {
            Some("A dedicated instruction which the network module does not support was executed.")
        }
        0xD278 => Some("The target network number of the SLMPREQ instruction is incorrect."),
        0xD279 => {
            Some("A dedicated instruction which the network module does not support was executed.")
        }
        0xD27A => Some("The own station number set in the UINI instruction is incorrect."),
        0xD280 => Some("The request command of transient transmission is incorrect."),
        0xD281 => Some("Transient reception failed."),
        0xD282 => Some("The receive queue is full."),
        0xD283 => Some("Transient transmission failed."),
        0xD284 => Some("The target execution module in the SLMP data is incorrect."),
        0xD2A0 => Some("The receive buffer is full."),
        0xD2A1 => Some("The send buffer is full."),
        0xD2A2 => {
            Some("Transmission completion wait timeout has occurred in transient transmission.")
        }
        0xD2A3 => Some("The data length in the transient transmission frame is incorrect."),
        0xD2A4 => Some("The header information in the transient transmission frame is incorrect."),
        0xD2A5 => {
            Some("The target station number in the transient transmission frame is incorrect.")
        }
        0xD2A6 => {
            Some("The request source number in the transient transmission frame is incorrect.")
        }
        0xD2A7 => Some("The header information in the transient transmission frame is incorrect."),
        0xD2A8 => Some("The header information in the transient transmission frame is incorrect."),
        0xD2A9 => {
            Some("The target network number in the transient transmission frame is incorrect.")
        }
        0xD2AA => {
            Some("The target station number in the transient transmission frame is incorrect.")
        }
        0xD2AB => Some(
            "The request source network number in the transient transmission frame is incorrect.",
        ),
        0xD2AC => Some(
            "The request source station number in the transient transmission frame is incorrect.",
        ),
        0xD2AD => Some("The data length in the transient transmission frame is incorrect."),
        0xD2AE => {
            Some("The target station number in the transient transmission frame is incorrect.")
        }
        0xD2AF => Some(
            "The own station number was specified as the target station number of transient transmission.",
        ),
        0xD2B0 => Some("Transient transmission failed."),
        0xD2B1 => Some("The receive queue is full."),
        0xD2E0 => Some("During execution of the IP communication test, the test was retried."),
        0xD2E1 => {
            Some("The IP communication test was completed with an error (no response to PING).")
        }
        0xD602 => Some("Parameter error"),
        0xD605 => Some("Parameter error"),
        0xD611 => Some("Parameter error (each station device range assignment error (RWw))"),
        0xD612 => Some("Parameter error (each station device range assignment error (RWw))"),
        0xD613 => Some("Parameter error (each station device range assignment error (RWr))"),
        0xD614 => Some("Parameter error (each station device range assignment error (RWr))"),
        0xD615 => Some("Parameter error (each station device range assignment error (RY))"),
        0xD616 => Some("Parameter error (each station device range assignment error (RY))"),
        0xD617 => Some("Parameter error (each station device range assignment error (RX))"),
        0xD618 => Some("Parameter error (each station device range assignment error (RX))"),
        0xD619 => Some("Parameter error"),
        0xD61A => Some("Parameter error"),
        0xD61B => Some("Parameter error (device overlap error (RWw))"),
        0xD61C => Some("Parameter error (device overlap error (RWr))"),
        0xD61D => Some("Parameter error (device overlap error (RY))"),
        0xD61E => Some("Parameter error (device overlap error (RX))"),
        0xD61F => Some("Parameter setting by the CCPASET instruction has failed."),
        0xD620 => Some("The transient data is incorrect."),
        0xD621 => Some("Parameter error"),
        0xD622 => Some("Parameter error (error in the total number of device stations)"),
        0xD623 => Some("Parameter error (link scan mode error)"),
        0xD624 => Some("Parameter error (constant link scan time setting error)"),
        0xD625 => Some("Parameter error (station-based block data assurance setting error)"),
        0xD626 => Some("Parameter error (loopback setting error)"),
        0xD628 => Some("Parameter error (station type error)"),
        0xD629 => Some("Parameter error (station number range error)"),
        0xD62A => Some("Parameter error (data link faulty station setting error)"),
        0xD62B => Some("Parameter error (output setting error during CPU STOP)"),
        0xD630 => {
            Some("Parameter setting of a local station by the CCPASET instruction has failed.")
        }
        0xD634 => Some("Parameter error (error in the number of submaster stations set)"),
        0xD635 => Some("Parameter error (submaster station error)"),
        0xD636 => Some("The UINI instruction was executed at a station other than local stations."),
        0xD637 => Some(
            "・The UINI instruction was executed at a station where the station number has been already set by parameter.・The UINI instruction was executed in a redundant system.",
        ),
        0xD638 => Some(
            "The station number set for the own station by the UINI instruction is already used for the other station.",
        ),
        0xD639 => Some(
            "After setting a station number with the UINI instruction, the instruction was executed again.",
        ),
        0xD63D => Some("Parameter error"),
        0xD63E => Some("Parameter error"),
        0xD641 => Some("Parameter error (IP address error)"),
        0xD701 => Some(
            "Temporary error invalid station setting/cancel or reserved station setting cancel/restoration was executed without specifying the target station.",
        ),
        0xD706 => Some(
            "Temporary error invalid station setting/cancel or reserved station setting cancel/restoration was executed from a local station.",
        ),
        0xD70B => Some(
            "Temporary error invalid station setting/cancel and reserved station cancel/restoration was executed simultaneously.",
        ),
        0xD720 => Some("Link startup/stop direction is incorrect."),
        0xD721 => Some(
            "Link start/stop was requested from another station during link start/stop processing.",
        ),
        0xD722 => Some(
            "Link start/stop was requested from the own station during link start/stop processing.",
        ),
        0xD723 => Some("System link start/stop was requested during link start/stop processing."),
        0xD724 => Some("Link startup/stop station specification is incorrect."),
        0xD725 => Some("System link start/stop was requested from a local station."),
        0xD726 => Some("The request command of transient transmission is incorrect."),
        0xD727 => Some(
            "Link start was requested from a station other than the station which had requested link stop.",
        ),
        0xD728 => Some(
            "Data link startup instruction was executed to the station which is performing data link.",
        ),
        0xD729 => Some(
            "Link stop of the own station was instructed in the station with no station number setting.",
        ),
        0xD731 => Some(
            "Forced master switching command was executed from a station other than the submaster station operating as a master operating station.",
        ),
        0xD740 => Some("Transient transmission failed."),
        0xD741 => Some("A station type error of the execution station was detected."),
        0xD742 => Some("Transient transmission failed."),
        0xD743 => Some("A station type error of the execution station was detected."),
        0xD744 => Some("Flash ROM clear failed."),
        0xD783 => Some("A transient transmission error was detected."),
        0xD784 => Some("A transient transmission error was detected."),
        0xD806 => Some("The receive queue is full."),
        0xD840 => Some(
            "Number of transient request exceeded the upper limit of simultaneously processable requests.",
        ),
        0xD841 => Some("The request data size of memory read/write command is out of range."),
        0xD842 => Some(
            "・Routing information to the destination network number is not registered.・In transient transmission, the number of relay to other networks exceeded seven.",
        ),
        0xD843 => Some(
            "The module operation mode is set to a mode in which transient transmission cannot be executed.",
        ),
        0xD844 => Some(
            "Incorrect frame is received.・Unsupported pre-conversion protocol・Unsupported frame type・Application header variable part・Application header HDS・Application header RTP・Read command not requiring response",
        ),
        0xD902 => Some("The online test data is incorrect."),
        0xD903 => Some("During execution of the communication test, the test was retried."),
        0xD905 => Some("A communication monitoring timeout has occurred in communication test."),
        0xD906 => Some("Transmission completion wait timeout has occurred in communication test."),
        0xD909 => Some("The header information of transient transmission is incorrect."),
        0xD90A => Some("During execution of the communication test, the test was retried."),
        0xD90B => {
            Some("The number of stations which communicates in the network is out of the range.")
        }
        0xD90C => Some("The target station specified for the communication test is incorrect."),
        0xD90D => Some("During execution of the cable test, the test was retried."),
        0xD90E => Some("The IP packet transfer function is not supported."),
        0xD90F => Some("During execution of the IP communication test, the test was retried."),
        0xD910 => Some("The IP address of the own station has not been set."),
        0xD911 => Some("The destination IP address setting of IP communication test is incorrect."),
        0xD912 => Some("Transient transmission failed."),
        0xD913 => Some("An error was detected in the network module."),
        0xD914 => Some("An error was detected in the network module."),
        0xD915 => Some("An error was detected in the network module."),
        0xD916 => Some("An error was detected in the network module."),
        0xD917 => Some("An error was detected in the network module."),
        0xD918 => Some(
            "The IP address of the standby system is set for \"Connected Station (Host)\" or \"Communication Target\" for the IP communication test.",
        ),
        0xDA00 => Some("An error was detected in the network module."),
        0xDA01 => Some("An error was detected in the network module."),
        0xDA10 => Some("An error was detected in the network module."),
        0xDA11 => Some("An error was detected in the network module."),
        0xDA12 => Some("An error was detected in the network module."),
        0xDA13 => Some("An error was detected in the network module."),
        0xDA14 => Some("An error was detected in the network module."),
        0xDA15 => Some("An error was detected in the network module."),
        0xDA16 => Some("An error was detected in the network module."),
        0xDA17 => Some("An error was detected in the network module."),
        0xDA19 => Some("An error was detected in the network module."),
        0xE006 => Some("The receive queue is full."),
        0xE102 => Some("The own station is set as a reserved station."),
        0xE103 => Some("The own station number set is out of the range of total stations."),
        0xE120 => Some("The UINI instruction was executed at the control station."),
        0xE121 => Some(
            "・The UINI instruction was executed when \"Parameter Editor\" is selected for \"Setting Method\" under \"Station Number\" of \"Required Settings\".・The UINI instruction was executed in a redundant system.",
        ),
        0xE122 => Some(
            "The station number set for the own station by the UINI instruction is already used for the other station.",
        ),
        0xE123 => Some(
            "After setting a station number with the UINI instruction, the instruction was executed again.",
        ),
        0xE160 => Some("'Link startup/stop direction' (SW0000) is not set properly."),
        0xE162 => Some(
            "Re-execution was attempted during the processing of cyclic transmission stop/restart.",
        ),
        0xE163 => Some(
            "Re-execution was attempted during the processing of cyclic transmission stop/restart.",
        ),
        0xE164 => Some(
            "Re-execution was attempted during the processing of cyclic transmission stop/restart.",
        ),
        0xE165 => Some(
            "'Link startup/stop station specification' (SW0001 to SW0008) is not set properly.",
        ),
        0xE166 => {
            Some("'Link startup/stop group specification' (SW0012 to SW0013) is not set properly.")
        }
        0xE170 => Some("An error was detected in the network module."),
        0xE171 => Some("An error was detected in the network module."),
        0xE172 => Some("An error was detected in the network module."),
        0xE173 => Some("During execution of the communication test, the test was retried."),
        0xE174 => Some("The maximum number of transmission completion signal retries was reached."),
        0xE175 => Some("No response has been returned within the communication monitoring time."),
        0xE176 => Some("Timeout has occurred without transmission completion."),
        0xE177 => Some("An error was detected in the network module."),
        0xE178 => Some("An error was detected in the network module."),
        0xE179 => Some("An error was detected in the network module."),
        0xE17A => Some("The response data have been received two times or more."),
        0xE17B => Some("An error was detected in the network module."),
        0xE17C => Some("The target station specified for the communication test is incorrect."),
        0xE17D => Some(
            "The IP address of the own station cannot be obtained when an IP communication test is performed.",
        ),
        0xE17E => Some(
            "The same numbers are not used for the first and second octets of the IP addresses set in the IP communication test destination setting in the network of the request source device, request destination device, and modules between them.",
        ),
        0xE17F => Some("An error was detected in the network module."),
        0xE180 => Some(
            "During execution of the cable test, the test was retried. (only when Ethernet cables are used)",
        ),
        0xE181 => Some("The IP packet transfer function is not supported."),
        0xE182 => Some("During execution of the IP communication test, the test was retried."),
        0xE183 => Some("Transient transmission failed."),
        0xE184 => Some("An error was detected in the network module."),
        0xE185 => Some("An error was detected in the network module."),
        0xE186 => Some(
            "The IP address of the standby system is set for \"Connected Station (Host)\" or \"Communication Destination Setting\" for the IP communication test.",
        ),
        0xE201 => Some("The same transient data have been received two times or more."),
        0xE203 => Some("The send buffer is full."),
        0xE204 => Some("The specified number of resends has been reached."),
        0xE205 => Some("The receive buffer is full."),
        0xE206 => Some("An error was detected in the network module."),
        0xE207 => Some(
            "Although the target station of transient transmission is connected in the same network, different network number is set.",
        ),
        0xE208 => {
            Some("The target station number specified for transient send/receive is out of range.")
        }
        0xE20A => Some("An error was detected in the network module."),
        0xE20B => {
            Some("In transient transmission, the number of relay to other networks exceeded seven.")
        }
        0xE20F => Some(
            "The target station number is set to zero in transient transmission using protocols such as SLMP.",
        ),
        0xE211 => Some(
            "When there was no control station, \"Specified Control Station\" was specified for transient transmission using protocols such as SLMP.",
        ),
        0xE212 => Some(
            "When there was no control station, \"Present Control Station\" was specified for transient transmission using protocols such as SLMP.",
        ),
        0xE213 => {
            Some("In transient transmission, timeout has occurred without transmission completion.")
        }
        0xE215 => Some("An error was detected in the network module."),
        0xE216 => Some("An error was detected in the network module."),
        0xE218 => Some("An error was detected in the network module."),
        0xE21B => Some(
            "Transient transmission was performed when the station number of the own station has not been set yet.",
        ),
        0xE21C => Some("An error was detected in the network module."),
        0xE21E => Some("An error was detected in the network module."),
        0xE21F => Some("An error was detected in the network module."),
        0xE221 => Some("An error was detected in the network module."),
        0xE222 => Some("An error was detected in the network module."),
        0xE223 => Some("An error was detected in the network module."),
        0xE224 => {
            Some("Attribute code set in the CC-Link transient request frame is out of range.")
        }
        0xE225 => Some("Access code set in the CC-Link transient request frame is out of range."),
        0xE226 => Some("An error was detected in the network module."),
        0xE228 => Some(
            "・The request command of transient transmission is incorrect.・The CC-Link IE Controller Network diagnostics was used for the network to which the relay receiving station belongs.",
        ),
        0xE229 => Some("The control station does not exist."),
        0xE22A => Some("A transient transmission error was detected."),
        0xE22B => Some("Baton pass has not been performed."),
        0xE22C => Some("A transient transmission error was detected."),
        0xE22D => Some("A transient transmission error was detected."),
        0xE236 => Some("The TTL of the IP data is incorrect."),
        0xE237 => Some("The IP address setting is not correctly set."),
        0xE241 => {
            Some("The hardware of the target network module for dedicated instruction has failed.")
        }
        0xE242 => {
            Some("The hardware of the target network module for dedicated instruction has failed.")
        }
        0xE243 => {
            Some("The hardware of the target network module for dedicated instruction has failed.")
        }
        0xE244 => {
            Some("The hardware of the target network module for dedicated instruction has failed.")
        }
        0xE245 => {
            Some("The hardware of the target network module for dedicated instruction has failed.")
        }
        0xE24F => Some(
            "When the dedicated instruction is executed, the target station number setting is not correct.",
        ),
        0xE251 => Some(
            "Transient data for the same dedicated instruction have been received two times or more.",
        ),
        0xE254 => Some(
            "The target station's CPU type specified for the dedicated instruction is out of range.",
        ),
        0xE255 => Some("The data size specified for the dedicated instruction is out of range."),
        0xE256 => Some(
            "The arrival monitoring time specified for the dedicated instruction is out of range.",
        ),
        0xE257 => {
            Some("The number of resends specified for the dedicated instruction is out of range.")
        }
        0xE258 => {
            Some("The network number specified for the dedicated instruction is out of range.")
        }
        0xE259 => Some("The channel used in the dedicated instruction is incorrect."),
        0xE25A => Some(
            "The modification specification specified for the UINI instruction is out of range.",
        ),
        0xE25B => {
            Some("The own station No. specified for the dedicated instruction is out of range.")
        }
        0xE262 => Some(
            "When the target station specified for the dedicated instruction is \"Group\" or \"All stations\", \"With arrival confirmation\" is specified for execution type. For the REQ instruction, the specified request type is incorrect.",
        ),
        0xE264 => Some(
            "Transmission did not completed after execution of the dedicated instruction, and timeout has occurred.",
        ),
        0xE265 => Some(
            "No response was received after execution of the dedicated instruction, and timeout has occurred.",
        ),
        0xE266 => Some("The SEND instruction was received from other network."),
        0xE267 => Some("The own station number was set as the target station number."),
        0xE268 => Some(
            "In the execution/abnormal completion type specification, the bit in the area fixed to 0 is turned on.",
        ),
        0xE269 => Some(
            "The request type or sub-request type specified in the REQ instruction is incorrect.",
        ),
        0xE26A => Some(
            "When there was no control station on the network, the dedicated instruction was executed specifying the specified control station or current control station.",
        ),
        0xE26C => Some("The channel specified is being used for another instruction."),
        0xE26D => Some("The channel specified is being used for event parameters."),
        0xE26E => Some("The device range specified for the ZNRD/ZNWR instruction is not correct."),
        0xE26F => Some("The device range specified for the ZNRD/ZNWR instruction is not correct."),
        0xE271 => Some(
            "The operation mode specified in the REQ instruction (remote RUN/STOP) is incorrect.",
        ),
        0xE272 => Some(
            "When the remote RUN is specified in the REQ instruction (remote RUN/STOP), the specified clear mode is not correct.",
        ),
        0xE273 => Some("The control data specified for the RRUN instruction is not correct."),
        0xE274 => Some("An error was detected in the network module."),
        0xE277 => Some("An error was detected in the network module."),
        0xE278 => Some("The request data size of transient transmission is out of range."),
        0xE279 => Some("The routing setting is not correctly set."),
        0xE27A => {
            Some("Dedicated instructions which cannot be executed simultaneously were executed.")
        }
        0xE27B => {
            Some("The target station type specification of the dedicated instruction is incorrect.")
        }
        0xE27C => Some("An error was detected in the network module."),
        0xE27D => Some("An error was detected in the network module."),
        0xE286 => Some("An error was detected in the network module."),
        0xE2A0 => Some("The receive buffer for the CC-Link dedicated instruction is full."),
        0xE2A1 => Some("The send buffer for the CC-Link dedicated instruction is full."),
        0xE2A2 => Some("The hardware of the network module has failed."),
        0xE2A3 => Some("The frame length (L) in the transient transmission frame is incorrect."),
        0xE2A4 => Some("The gate count (GCNT) in the transient transmission frame is incorrect."),
        0xE2A5 => Some(
            "The destination station number (DA) in the transient transmission frame is incorrect.",
        ),
        0xE2A6 => {
            Some("The source station number (SA) in the transient transmission frame is incorrect.")
        }
        0xE2A7 => Some(
            "The destination application type (DAT) in the transient transmission frame is incorrect.",
        ),
        0xE2A8 => Some(
            "The source application type (SAT) in the transient transmission frame is incorrect.",
        ),
        0xE2A9 => Some(
            "The destination network number (DNA) in the transient transmission frame is incorrect.",
        ),
        0xE2AA => Some(
            "The destination station number (DS) in the transient transmission frame is incorrect.",
        ),
        0xE2AB => Some(
            "The source network number (SNA) in the transient transmission frame is incorrect.",
        ),
        0xE2AC => {
            Some("The source station number (SS) in the transient transmission frame is incorrect.")
        }
        0xE2AD => Some("The data length (L1) in the transient transmission frame is incorrect."),
        0xE2AE => Some(
            "The destination station number (DA) in the transient transmission frame of the received data matches the own station, but the destination network number (DNA) or the destination station number (DS) does not match the own station.",
        ),
        0xE2AF => Some(
            "The own station number was set as the target station number of the CC-Link dedicated instruction.",
        ),
        0xE2B0 => Some("An error was detected in the network module."),
        0xE501 => Some("An error was detected in the network module."),
        0xE502 => Some("An error was detected in the network module."),
        0xE503 => Some("An error was detected in the network module."),
        0xE504 => Some(
            "Transient transmission (dedicated instruction, engineering tool connection) was executed while the own station did not perform baton pass.",
        ),
        0xE505 => Some(
            "Transient transmission (dedicated instruction, engineering tool connection) was executed with the own station number duplicated.",
        ),
        0xE521 => Some("An error was detected in the network module."),
        0xE5F0 => Some(
            "Transient transmission (dedicated instruction, engineering tool connection) was executed while the target station did not perform baton pass.",
        ),
        0xE5F1 => Some("The target station number of transient transmission is already in use."),
        0xE5F8 => Some(
            "There is a station that does not support the IP packet transfer function on the communication path when the IP packet transfer function is used.",
        ),
        0xE840 => Some(
            "Number of transient request exceeded the upper limit of simultaneously processable requests.",
        ),
        0xE841 => Some("The request data size of memory read/write command is out of range."),
        0xE842 => Some(
            "・Routing information to the destination network number is not registered.・In transient transmission, the number of relay to other networks exceeded seven.",
        ),
        0xE843 => Some(
            "The module operation mode is set to a mode in which transient transmission cannot be executed.",
        ),
        0xE844 => Some(
            "Incorrect frame is received.・Unsupported pre-conversion protocol・Unsupported frame type・Application header variable part・Application header HDS・Application header RTP・Read command not requiring response",
        ),
        0xEA00 => Some("An error was detected in the network module."),
        0xEA01 => Some("An error was detected in the network module."),
        _ => None,
    }
}

/// Return the Japanese error detail/cause message for an SLMP end code.
pub fn end_code_message_ja(end_code: u16) -> Option<&'static str> {
    match end_code {
        0x1080 => Some("フラッシュROMへの書込み回数が10万回を超えた。"),
        0x1120 => Some("シーケンサ電源ON/リセット時の時計設定に失敗した。"),
        0x1124 => Some(
            "・デフォルトゲートウェイの設定値に誤りがある。・ゲートウェイIPアドレスの設定値に誤りがある。・デフォルトゲートウェイ/ゲートウェイIPアドレス(サブネットマスク後のネットワークアドレス)が自ノードのIPアドレスのネットワークアドレスと異なる。",
        ),
        0x1128 => Some("ポート番号に誤りがある。"),
        0x1129 => Some("相手機器のポート番号の設定値に誤りがある。"),
        0x112C => Some("全局指定による要求に失敗した。"),
        0x112D => Some(
            "”基本設定”の”相手機器接続構成設定”で設定した接続する機器のIPアドレスの設定に誤りがある状態で，その機器に対して送信を行った。",
        ),
        0x112E => Some("オープン処理で，コネクションが確立されなかった。"),
        0x1133 => Some("ソケット通信または固定バッファによる交信でレスポンス送信に失敗した。"),
        0x1134 => Some(
            "TCP/IPの交信で，TCP ULPタイムアウトエラーが発生した。(相手機器からACKが返されない)",
        ),
        0x1152 => Some(
            "・IPアドレスの設定値に誤りがある。・Ethernet搭載ユニットのポート1とポート2のIPアドレスが重複している。",
        ),
        0x1155 => Some(
            "・TCP/IP交信で指定したコネクションがすでにクローズされている。・オープン処理が実施されていない。",
        ),
        0x1157 => Some(
            "・UDP/IP交信で指定したコネクションがすでにクローズされている。・オープン処理が実施されていない。",
        ),
        0x1158 => Some(
            "・受信バッファまたは送信バッファが不足している。・相手機器のウィンドウサイズが不足している。",
        ),
        0x1165 => Some("UDP/IPによる送信が正常に行えなかった。"),
        0x1166 => Some("TCP/IPによる送信が正常に行えなかった。"),
        0x1167 => Some("未送信のデータがあるが，残りのデータを送信できなかった。"),
        0x1180 => Some(
            "・A系IPアドレス，B系IPアドレス，制御系IPアドレスに重複が存在する。・A系IPアドレス，B系IPアドレス，制御系IPアドレスのネットワークアドレスが異なっている。",
        ),
        0x1800 => Some("ネットワークの接続異常を検出した。"),
        0x1801 => Some("送信相手機器のIPアドレスが取得できない。"),
        0x1811 => Some("CPUユニットの異常を検出した。"),
        0x1830 => {
            Some("トランジェント伝送(リンク専用命令)の受信要求数が，同時処理可能な上限を超過した。")
        }
        0x1845 => Some(
            "トランジェント伝送(リンク専用命令)の処理数が多すぎてトランジェント伝送が実行できない。",
        ),
        0x1860 => Some(
            "通信回線の異常またはCC-Link IEコントローラネットワーク搭載ユニットの異常によりバトンパスが停止した。",
        ),
        0x1D01 => Some(
            "・マスタ局の”基本設定”の”ネットワーク構成設定”にある”ネットワーク同期通信設定”と管理されているデバイス局のネットワーク同期通信の設定(同期有無)とで不一致となっているデバイス局を検出した。・マスタ局の”基本設定”の”ネットワーク構成設定”にある”ネットワーク同期通信設定”で”同期する”に設定したデバイス局がネットワーク同期通信に対応していない機器である。",
        ),
        0x1D10 => Some("サイクリック伝送抜けが発生した。"),
        0x1D20 => {
            Some("CC-Link IEフィールドネットワークの同期デバイスで正常に通信ができなくなった。")
        }
        0x1F07 => Some("”シンプル機器通信設定”で設定したプロトコルデータに誤りがある。"),
        0x20E0 => Some("CPUユニットと通信できない。"),
        0x2160 => Some("IPアドレスの重複を検出した。"),
        0x2220 => Some(
            "・シンプルCPU通信機能に対応していないファームウェアバージョンのネットワークユニットを使用している。・シンプルCPU通信設定が65以上設定されている。・パラメータの内容が壊れている。・パラメータでリンク点数拡張設定を拡張する設定にしているが，CPUユニット，ネットワークユニットがリンク点数拡張設定に未対応である。",
        ),
        0x2221 => Some(
            "・パラメータの設定値が使用可能な範囲を超えている。・自局に設定されたネットワークNo.が，管理局に設定されたネットワークNo.と異なる。・自局は拡張モードに設定しているが，管理局が通常モードである。もしくは，自局は通常モードに設定しているが，管理局が拡張モードである。・”応用設定”の”リンク点数拡張設定”を，自局は”拡張する”に設定しているが，管理局が”拡張しない”である。もしくは，自局は”拡張しない”に設定しているが，管理局が”拡張する”である。・パラメータの設定値が使用可能な範囲を超えている。または，マスタ局のパラメータでデバイス局のリセットが必要な項目が変更された。",
        ),
        0x2250 => Some(
            "CPUユニットに格納されているプロトコル設定データが，Ethernet搭載ユニット用ではない。",
        ),
        0x24C0 => Some("システムバスの異常を検出した。"),
        0x24C1 => Some("システムバスの異常を検出した。"),
        0x24C2 => Some("システムバスの異常を検出した。"),
        0x24C3 => Some("システムバスの異常を検出した。"),
        0x24C6 => Some("システムバスの異常を検出した。"),
        0x2600 => {
            Some("サイクリック処理が，次のユニット間同期周期の開始タイミングまでに完了できない。")
        }
        0x2610 => Some("ユニット間同期信号の異常(同期外れ)を検出した。"),
        0x3000 => Some(
            "・”システムパラメータ”画面の[ユニット間同期設定]タブにある”ユニット間同期設定”で同期対象として設定しているユニットに，下記の設定をしている。・\"必須設定\"の\"局番設定\"の\"局番設定方法\"を\"プログラムで設定\"に設定・\"必須設定\"の\"パラメータ設定方法\"の\"基本設定/応用設定の設定方法\"を\"プログラムで設定\"に設定・\"必須設定\"の\"局種別設定\"の\"局種別\"を\"サブマスタ局\"に設定・\"基本設定\"の\"伝送路形式設定\"を\"リング接続\"に設定・\"応用設定\"の\"サイクリック補助設定\"の\"リンクスキャンモード\"を\"コンスタントリンクスキャン設定\"，または\"シーケンススキャン同期設定\"に設定・\"基本設定\"の\"ネットワーク構成設定\"に，\"局種別\"が\"サブマスタ局\"の局を設定・“基本設定”の”ネットワーク構成設定”にある”ネットワーク同期通信設定”で”同期する”に設定しているデバイス局が存在するが，下記の状態になっている。・”システムパラメータ”画面の[ユニット間同期設定]タブにある”ユニット間同期設定”で，マスタ・ローカルユニットが対象ユニットに設定されていない。・ユニット間同期機能を使用できないCPUユニットが管理CPUになっている。",
        ),
        0x3001 => Some(
            "・既に同じ局番の局が同一ネットワーク上に存在することを検出した。・同一ネットワーク上に管理局が複数存在することを検出した。・既に同じ局番の局が同一ネットワーク上に存在することを検出した。・同一ネットワーク上にマスタ局，サブマスタ局が複数存在することを検出した。・同一ネットワーク上にCC-Link IEコントローラネットワーク(Ethernetケーブル)の局が存在することを検出した。",
        ),
        0x3004 => Some(
            "”基本設定”の”ネットワーク構成設定”にある”RWw/RWr設定”で，安全局に設定した点数がシステムで使用する点数未満になっている。システムで使用する点数: ”応用設定”の”安全通信設定”にある”安全プロトコルバージョン”で設定したバージョンの安全通信で使用する点数。",
        ),
        0x3005 => Some("マスタ局とサブマスタ局のパラメータが不一致となっている。"),
        0x3006 => Some("二重化システムの局にペアリング設定がされていない。"),
        0x3007 => Some("二重化システム以外の局にペアリング設定がされている。"),
        0x3008 => Some(
            "・二重化増設ベース構成において，増設ベース上のユニット形名に”RJ71EN71(E+E)”，”RJ71EN71(Q)”以外を選択している。・二重化システムで，ユニット形名に”RJ71GP21-SX”または”RJ71GP21S-SX”を選択している。・二重化システム以外で，ユニット形名に”RJ71GP21-SX(R)”または”RJ71GP21S-SX(R)”を選択している。・二重化システムで，ユニット形名に”RJ71GF11-T2”を選択している。・二重化システム以外で，ユニット形名に”RJ71GF11-T2(MR)”，”RJ71GF11-T2(SR)”または”RJ71GF11-T2(LR)”を選択している。",
        ),
        0x3019 => Some(
            "・二重化システムにおいて基本ベースに装着時，”応用設定”の”二重化設定使用有無”が”使用しない”に設定されている。・二重化増設ベース構成において増設ベースに装着時，”応用設定”の”二重化設定使用有無”が”使用する”に設定されている。",
        ),
        0x301A => Some(
            "二重化増設ベース構成において増設ベースに装着時，”応用設定”の”動的ルーチング設定”が”有効”に設定されている。",
        ),
        0x301B => Some(
            "二重化増設ベース構成において増設ベースに装着時，”応用設定”の”IPパケット中継機能使用有無”が”使用する”に設定されている。",
        ),
        0x301C => Some(
            "・”シンプル機器通信設定”で設定したプロトコルデータに誤りがある。・本ファームウェアバージョンでは対応していないパケットを使用した。",
        ),
        0x301D => Some("本ファームウェアバージョンでは対応していない機能を使用した。"),
        0x301E => Some(
            "ポート1またはポート2の”シンプル機器通信設定”で設定している”リソース設定”の設定が誤っている。",
        ),
        0x301F => Some(
            "”安全通信設定”で，ネットワークユニットまたはCPUユニット(安全CPUまたはSIL2プロセスCPU)が対応していない安全プロトコルバージョンが設定されている。",
        ),
        0x3020 => Some("システムポート番号の値が範囲外になっている。"),
        0x3022 => Some("パラメータで設定した相手機器と交信相手の合計が64を超えている。"),
        0x3023 => Some(
            "”シンプル機器通信設定”で設定した”ポート番号 自局”が，他機能で設定した値と重複している。",
        ),
        0x3026 => Some(
            "本ユニットが”シンプル機器通信設定”で設定している”リソース設定”の”拡張2”に対応していない。",
        ),
        0x3040 => Some("専用命令の応答データの作成ができない。"),
        0x3060 => Some("送受信データサイズが許容範囲を超えている。"),
        0x31D0 => Some(
            "・”応用設定”の”安全通信設定”で，安全プロトコルバージョン2に対応していない交信相手に安全プロトコルバージョン2が選択されている。・安全プロトコルバージョン2を設定している同一の交信相手との安全コネクション数が17コネクション以上設定されている。",
        ),
        0x3600 => Some("ユニット間同期周期設定でマスタ局の設定とで不一致となっている。"),
        0x3601 => Some(
            "マスタ局の”基本設定”の”ネットワーク構成設定”にある”ネットワーク同期通信設定”と自局のユニット間同期対象ユニットの選択とで不一致となっている。",
        ),
        0x3602 => Some("ネットワーク間のユニット間同期周期が異常になった。"),
        0x3C00 => Some(
            "・ハードウェアの異常を検出した。・二重化システム構成の場合，トラッキングケーブルが接続されていない状態で，制御系を電源OFFまたはリセットした。",
        ),
        0x3C01 => Some(
            "・ハードウェアの異常を検出した。・二重化システム構成の場合，トラッキングケーブルが接続されていない状態で，制御系を電源OFFまたはリセットした。",
        ),
        0x3C02 => Some(
            "・ハードウェアの異常を検出した。・二重化システム構成の場合，トラッキングケーブルが接続されていない状態で，制御系を電源OFFまたはリセットした。",
        ),
        0x3C03 => Some(
            "・ハードウェアの異常を検出した。・二重化システム構成の場合，トラッキングケーブルが接続されていない状態で，制御系を電源OFFまたはリセットした。",
        ),
        0x3C0F => Some("ハードウェアの異常を検出した。"),
        0x3C10 => Some(
            "・ハードウェアの異常を検出した。・対応していない機能を使用した。(Ethernetケーブル使用時)",
        ),
        0x3C11 => Some("ハードウェアの異常を検出した。"),
        0x3C13 => Some("ハードウェアの異常を検出した。"),
        0x3C14 => Some("ハードウェアの異常を検出した。"),
        0x3C2F => Some("メモリの異常を検出した。"),
        0x3E00 => Some("ネットワークユニットの異常を検出した。"),
        0x3E01 => Some("自局のネットワーク種別が想定外の設定となっている。"),
        0xC001 => Some(
            "・イニシャル処理時の，自ユニットのIPアドレスの設定値に誤りがある。・ルータ中継機能を使用時，サブネットマスクフィールドの設定値に誤りがある。",
        ),
        0xC012 => {
            Some("オープンしているコネクションで使用しているポート番号を設定した。(TCP/IPの場合)")
        }
        0xC013 => {
            Some("オープンしているコネクションで使用しているポート番号を設定した。(UDP/IPの場合)")
        }
        0xC015 => Some(
            "・オープン処理時の，相手機器のIPアドレスの設定値に誤りがある。・専用命令の相手機器IPアドレスの設定に誤りがある。",
        ),
        0xC016 => Some("ペアリングオープンのコネクションは，すでにオープン処理されている。"),
        0xC018 => Some("相手機器IPアドレスの設定に誤りがある。"),
        0xC020 => Some("送受信データ長が許容範囲を超えている。"),
        0xC021 => Some(
            "固定バッファ，ランダムアクセスバッファによる送信に対して，異常終了のレスポンスを受信した。",
        ),
        0xC022 => Some(
            "・レスポンス監視タイマ値以内に，レスポンスを受信できなかった。・レスポンス待ち中に該当コネクションがクローズされた。",
        ),
        0xC024 => Some(
            "・交信手順が”通信プロトコル”のコネクションにて，固定バッファ交信，またはランダムアクセス用バッファ交信を実施した。・交信手段が”固定バッファ(手順あり)”または，”固定バッファ(手順なし)”のコネクションにて，通信プロトコルによる交信を実施した。",
        ),
        0xC025 => Some(
            "CONOPEN/OPEN命令または入出力信号によるオープン処理時に，使用用途設定エリアの指定に誤りがある。",
        ),
        0xC026 => Some("通信プロトコル設定データの読出し/書込み/照合中に異常が発生した。"),
        0xC027 => Some("ソケット通信の伝文送信に失敗した。"),
        0xC028 => Some("固定バッファの伝文送信に失敗した。"),
        0xC029 => Some(
            "・コントロールデータの内容がおかしい。・オープン設定パラメータが未設定なのに，オープン設定パラメータでのオープンを指定した。",
        ),
        0xC035 => Some("レスポンス監視タイマ値以内に，相手機器の生存確認ができなかった。"),
        0xC040 => Some(
            "・レスポンス監視タイマ値以内に，すべてのデータを受信できなかった。・データ長分のデータを受信できなかった。・TCP/IPレベルで分割された伝文の残りを，レスポンス監視タイマ値以内に受信できなかった。",
        ),
        0xC050 => Some(
            "Ethernet搭載ユニットの交信データコードが”ASCII”に設定されている場合に，バイナリ変換できないASCIIコードのデータを受信した。",
        ),
        0xC051 => Some(
            "・SLMP伝文の，CPUユニットのワード単位でのデバイス読出し/書込み点数が許容範囲外である。・SLMP伝文の，ロングカウンタに対する書込み点数が2ワード単位でない。",
        ),
        0xC052 => Some(
            "SLMP伝文の，CPUユニットのビット単位でのデバイス読出し/書込み点数が許容範囲外である。",
        ),
        0xC053 => Some(
            "SLMP伝文の，CPUユニットのビット単位でのランダムデバイス読出し/書込み点数が許容範囲外である。",
        ),
        0xC054 => Some(
            "SLMP伝文の，CPUユニットのワード・ダブルワード単位でのランダムデバイス読出し/書込み点数が許容範囲外である。",
        ),
        0xC055 => Some("SLMP伝文の，ファイルの読出し/書込みサイズが許容範囲外である。"),
        0xC056 => Some("最大アドレスを超える書込みおよび読出し要求である。"),
        0xC057 => {
            Some("SLMP伝文の要求データ長が，キャラクタ部(テキストの一部)のデータ数と合わない。")
        }
        0xC058 => Some(
            "SLMP伝文の，ASCII－バイナリ変換後の要求データ長が，キャラクタ部(テキストの一部)のデータ数と合わない。",
        ),
        0xC059 => Some(
            "・SLMP伝文のコマンド，サブコマンドの指定に誤りがある。・対象機器が未サポートの機能を実行した。",
        ),
        0xC05A => Some(
            "SLMP伝文で指定されたデバイスに対して，Ethernet搭載ユニットから読出し/書込みができない。",
        ),
        0xC05B => Some(
            "SLMP伝文で指定されたデバイスに対して，Ethernet搭載ユニットから読出し/書込みができない。",
        ),
        0xC05C => Some(
            "・受信したSLMP伝文の要求データに誤りがある。・iQSS機能実行時の通信設定の設定値が範囲外である。・iQSS機能実行時，対象機器に設定できない通信設定項目を設定した。・iQSS機能実行時，対象機器で設定必須の項目が未設定である。",
        ),
        0xC05D => Some(
            "SLMP伝文”モニタ登録/解除”コマンドによるモニタ登録を行う前に，”モニタ要求”コマンドを受信した。",
        ),
        0xC05E => Some(
            "・SLMP伝文をEthernet搭載ユニットが受信し，アクセス先から応答が返るまでの時間が，SLMPコマンドに設定された監視タイマの値を超えた。・応答伝文のないコマンドを指定したSLMPの要求伝文を，他ネットワークNo.のユニットをアクセス先として送信した。",
        ),
        0xC05F => Some("SLMP伝文で指定されたアクセス先には，実行できない要求である。"),
        0xC060 => Some("SLMP伝文の，ビットデバイスに対する要求内容に誤りがある。"),
        0xC061 => Some(
            "・SLMP伝文の要求データ長が，キャラクタ部(テキストの一部)のデータ数と合わない。・ラベル書込みコマンドで指定した書込みデータの長さが偶数バイトでない。・iQSS機能実行時，異常なフレームを受信した。",
        ),
        0xC070 => {
            Some("SLMP伝文で指定されたアクセス先は，デバイスメモリの拡張指定に対応していない。")
        }
        0xC071 => Some("SLMP伝文のR/Q/QnACPU以外に対するデバイス読出し/書込み点数が範囲外である。"),
        0xC072 => Some(
            "SLMP伝文の要求内容に誤りがある。(ワードデバイスに対するビット単位の読出し/書込みなど)",
        ),
        0xC073 => Some(
            "SLMP伝文のアクセス先がサポートしていない要求である。(R/Q/QnACPU以外に対するダブルワードアクセス点数の指定があるなど)",
        ),
        0xC075 => Some("ラベルアクセスにおける要求データ長が範囲外である。"),
        0xC081 => Some(
            "再イニシャル実行に伴うEthernet搭載ユニットの終了処理が行われており，リンク専用命令交信の到達確認ができない。",
        ),
        0xC083 => Some("リンク専用命令交信で，交信処理が異常終了した。"),
        0xC084 => Some("リンク専用命令交信で，交信処理が異常終了した。"),
        0xC085 => {
            Some("リンク専用命令SENDで指定した対象局格納チャンネルは，対象局にて現在使用中である。")
        }
        0xC0B2 => Some(
            "MELSOFT接続，リンク専用命令，SLMPの中継局/相手局で受信バッファに空きがない，または送信バッファに空きがない。(送信・受信バッファフルエラー)",
        ),
        0xC0B3 => Some("CPUユニットから処理できない要求があった。"),
        0xC0B6 => Some("専用命令で指定されたチャンネルが範囲外である。"),
        0xC0BA => {
            Some("CONCLOSE/CLOSE命令によるクローズ処理中のため，送信要求を受け付けられない。")
        }
        0xC0C4 => Some("通信中にUINI命令が実行された。"),
        0xC0D0 => Some("リンク専用命令のデータ長の指定に誤りがある。"),
        0xC0D1 => Some("リンク専用命令の再送回数の指定に誤りがある。"),
        0xC0D3 => Some("他ネットワークとの交信の中継局数が，許容数を超えた。"),
        0xC0D4 => Some("他ネットワークとの交信の中継局数が，許容数を超えた。"),
        0xC0D5 => Some("リンク専用命令のリトライ回数の指定に誤りがある。"),
        0xC0D6 => Some("リンク専用命令のネットワークNo./局番の指定に誤りがある。"),
        0xC0D7 => Some("イニシャル処理が完了していない状態で送信処理が行われた。"),
        0xC0D8 => Some("指定したブロック数が範囲を越えている。"),
        0xC0D9 => Some("SLMP伝文のサブコマンドの指定に誤りがある。"),
        0xC0DA => Some("交信タイムチェック時間以内に，PINGテストの応答を受信できなかった。"),
        0xC0DB => Some("PINGテストする対象先のIPアドレス/ホスト名に誤りがある。"),
        0xC0DE => Some("指定された到達監視時間以内にデータを受信できなかった。"),
        0xC101 => Some("DNSサーバから応答を受信できなかった。"),
        0xC1A2 => Some(
            "・要求に対する応答を受信できなかった。・トランジェント伝送で，他のネットワークへの中継回数が7回を超えた。",
        ),
        0xC1A4 => Some(
            "・SLMP伝文のコマンド，サブコマンド，要求先ユニットI/O番号の指定に誤りがある。・ERRCLEAR命令で指定したクリア機能指定に誤りがある。・ERRRD命令で指定した読出し対象情報指定に誤りがある。・RJ71EN71のEthernetポートに直結接続してEthernet診断，CC-Link IEフィールドネットワーク診断またはCCLinkIEコントローラネットワーク診断を使用しようとした。・対象機器が未サポートの機能を実行した。",
        ),
        0xC1A5 => Some("対象局またはクリア対象の指定に誤りがある。"),
        0xC1A6 => Some("コネクションNo.の指定に誤りがある。"),
        0xC1A7 => Some("ネットワークNo.の指定に誤りがある。"),
        0xC1A8 => Some("局番の指定に誤りがある。"),
        0xC1A9 => Some("デバイスNo.の指定に誤りがある。"),
        0xC1AA => Some("デバイス名の指定に誤りがある。"),
        0xC1AC => Some("再送回数の指定に誤りがある。"),
        0xC1AD => Some("データ長の指定に誤りがある。"),
        0xC1AF => Some("ポート番号の指定に誤りがある。"),
        0xC1B0 => Some("指定されたコネクションはすでにオープン処理が完了している。"),
        0xC1B1 => Some("指定されたコネクションはオープン処理が完了していない。"),
        0xC1B2 => Some(
            "CONOPEN/CONCLOSE/OPEN/CLOSE命令でオープン/クローズ処理を実行中のコネクションを指定して処理を行った。",
        ),
        0xC1B3 => Some("指定されたチャンネルは他の送受信命令が実行中である。"),
        0xC1B4 => Some("到達時間の指定に誤りがある。"),
        0xC1B8 => Some("データを受信していないチャンネルに対してRECV命令を実行した。"),
        0xC1B9 => Some("指定されたコネクションに対してCONOPEN/OPEN命令を実行できない。"),
        0xC1BA => Some("イニシャル未完了状態で専用命令を実行した。"),
        0xC1BB => Some("リンク専用命令の対象局CPU種別に誤りがある。"),
        0xC1BC => Some("リンク専用命令の対象ネットワークNo.に誤りがある。"),
        0xC1BD => Some("リンク専用命令の対象局番に誤りがある。"),
        0xC1BE => Some("専用命令のコマンドコードに誤りがある。"),
        0xC1BF => Some("専用命令の使用チャンネルに誤りがある。"),
        0xC1C0 => Some("トランジェントデータに誤りがある。"),
        0xC1C1 => Some("トランジェントデータに誤りがある。"),
        0xC1C2 => Some("専用命令で二重に受信した。"),
        0xC1C4 => Some("リンク専用命令の到達確認が異常完了した。"),
        0xC1C5 => Some("対象局が未サポートの専用命令を実行した。"),
        0xC1C6 => Some("専用命令の実行・異常時完了タイプの設定に誤りがある。"),
        0xC1C7 => Some("REQ命令のリクエストタイプの設定に誤りがある。"),
        0xC1C8 => Some("使用中のチャンネルを指定して専用命令を実行した。"),
        0xC1C9 => Some("ZNRD/ZNWR命令のデバイス指定が誤っている。"),
        0xC1CA => Some("ZNRD/ZNWR命令のデバイス指定が誤っている。"),
        0xC1CB => Some("トランジェントデータに誤りがある。"),
        0xC1CC => Some("SLMPSND命令で許容範囲を超えるデータ長の応答を受信した。"),
        0xC1CD => Some("SLMPSND命令の伝文送信に失敗した。"),
        0xC1D0 => Some("専用命令の要求先ユニットI/O番号に誤りがある。"),
        0xC1D2 => Some("リンク専用命令の対象局IPアドレスの設定に誤りがある。"),
        0xC1D3 => Some("コネクションの交信手段に対応していない専用命令を実行した。"),
        0xC200 => Some("リモートパスワードに誤りがある"),
        0xC201 => Some("交信に使ったポートがリモートパスワードのロック状態である。"),
        0xC202 => {
            Some("他局アクセスを行ったときに，リモートパスワードのアンロック処理ができなかった。")
        }
        0xC203 => Some("リモートパスワードのチェックで異常が発生した。"),
        0xC204 => Some("リモートパスワードのアンロック処理を要求した機器と異なる。"),
        0xC205 => {
            Some("他局アクセスを行ったときに，リモートパスワードのアンロック処理ができなかった。")
        }
        0xC207 => Some("ファイル名の文字数が長すぎる。"),
        0xC208 => Some("パスワード長が範囲外である。"),
        0xC400 => Some("通信プロトコル準備未完了時にECPRTCL命令を実行した。"),
        0xC401 => {
            Some("ECPRTCL命令で指定したプロトコル番号が，Ethernet搭載ユニットの登録されていない。")
        }
        0xC402 => Some(
            "Ethernet搭載ユニットに登録したプロトコル設定データに異常があり，ECPRTCL命令が実行できない。",
        ),
        0xC403 => Some("専用命令が同時に実行された。"),
        0xC404 => Some("ECPRTCL命令で実行中のプロトコルがキャンセルされた。"),
        0xC405 => Some("ECPRTCL命令で指定したプロトコル番号に誤りがある。"),
        0xC406 => Some("ECPRTCL命令のプロトコル連続実行数に誤りがある。"),
        0xC407 => Some("ECPRTCL命令で指定したコネクションNo.に誤りがある。"),
        0xC408 => Some("ECPRTCL命令の通信プロトコルの送信処理に異常が発生した。"),
        0xC410 => Some("ECPRTCL命令の受信待ち時間がタイムアウトした。"),
        0xC412 => Some("ASCII-バイナリ変換できないデータを受信した。"),
        0xC413 => Some("通信プロトコルで受信したデータの桁数が不足している。"),
        0xC414 => Some("通信プロトコルで受信したデータの桁数に誤りがある。"),
        0xC417 => Some("通信プロトコルで受信したデータのデータ長，またはデータ数が範囲外。"),
        0xC420 => Some("プロトコル設定データの書込みに失敗した。"),
        0xC421 => {
            Some("フラッシュROM書込み回数がオーバーしているユニットに対して書込みを要求した。")
        }
        0xC430 => Some("ECPRTCL命令実行中にプロトコル設定データの書込みが行われた。"),
        0xC431 => Some("ECPRTCL命令実行中にコネクションのクローズ処理が行われた。"),
        0xC440 => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC441 => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC442 => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC443 => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC444 => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC445 => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC446 => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC447 => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC448 => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC449 => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC44A => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC44B => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC44C => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC44D => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC44E => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC44F => Some("Ethernet診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xC610 => Some("ユニットの処理が異常完了した。"),
        0xC611 => Some("ユニットの処理が異常完了した。"),
        0xC612 => Some("ユニットの処理が異常完了した。"),
        0xC613 => Some("ユニットの処理が異常完了した。"),
        0xC614 => Some("ユニットの処理が異常完了した。"),
        0xC615 => Some("ユニットの処理が異常完了した。"),
        0xC616 => Some("FTPサーバへの制御ポートの接続に失敗した。"),
        0xC617 => Some("FTPサーバへの制御ポートの切断に失敗した。"),
        0xC618 => Some("FTPサーバへのログインに失敗した。"),
        0xC619 => Some("FTPサーバへのFTPコマンドの実行に失敗した。"),
        0xC620 => Some("FTPサーバへのデータ転送ポートの接続に失敗した。"),
        0xC621 => Some("FTPサーバへのデータ転送ポートの切断に失敗した。"),
        0xC622 => Some("ファイル転送中にエラーが発生した。"),
        0xC623 => Some("FTPサーバから応答を受信できなかった。"),
        0xC700 => Some("ユニットの処理が異常完了した。"),
        0xC701 => Some(
            "IPパケット中継機能による交信にて，IPアドレス(ネットワークNo.)の設定が誤っている。",
        ),
        0xC702 => Some("IPパケット中継機能による交信にて，IPアドレス(局番)の設定が誤っている。"),
        0xC703 => Some("IPパケット中継機能による交信にて，宛先IPアドレス(上位)が誤っている。"),
        0xC704 => Some("IPパケット中継機能による交信にて，宛先IPアドレス(下位)が誤っている。"),
        0xC705 => Some("ユニットの処理が異常完了した。"),
        0xC706 => Some("ユニットの処理が異常完了した。"),
        0xC707 => Some("ユニットの処理が異常完了した。"),
        0xC708 => Some(
            "IPパケット中継機能による交信にて，Ethernet機器と接続しているEthernet搭載ユニットの”応用設定”の”IPパケット中継設定”で”IPパケット中継機能使用有無”が”使用しない”になっている",
        ),
        0xC709 => Some("MELSOFT直結接続で交信異常が発生した。"),
        0xC810 => Some(
            "リモートパスワード認証が必要なアクセス時に，リモートパスワードのパスワード認証に失敗した。正しいパスワードを設定して再度実行してください。",
        ),
        0xC811 => Some(
            "リモートパスワード認証が必要なアクセス時に，リモートパスワードのパスワード認証に失敗した。1分後に正しいパスワードを設定して再度実行してください。",
        ),
        0xC812 => Some(
            "リモートパスワード認証が必要なアクセス時に，リモートパスワードのパスワード認証に失敗した。5分後に正しいパスワードを設定して再度実行してください。",
        ),
        0xC813 => Some(
            "リモートパスワード認証が必要なアクセス時に，リモートパスワードのパスワード認証に失敗した。15分後に正しいパスワードを設定して再度実行してください。",
        ),
        0xC814 => Some(
            "リモートパスワード認証が必要なアクセス時に，リモートパスワードのパスワード認証に失敗した。60分後に正しいパスワードを設定して再度実行してください。",
        ),
        0xC815 => Some(
            "リモートパスワード認証が必要なアクセス時に，リモートパスワードのパスワード認証に失敗した。60分後に正しいパスワードを設定して再度実行してください。",
        ),
        0xC816 => Some("セキュリティ機能が動作し，リモートパスワード認証不可状態である。"),
        0xC840 => Some("トランジェントの要求数が，配送処理で同時処理可能な上限を超過した。"),
        0xC842 => Some("宛先ネットワークNo.へ到達するためのルーチング設定がされていない。"),
        0xC843 => Some("設定されているネットワーク種別では実行できないリンク専用命令を実行した。"),
        0xC844 => Some(
            "異常なフレームを受信した。・未対応変換前プロトコル・未対応フレームタイプ・アプリケーションヘッダ可変部・アプリケーションヘッダHDS・アプリケーションヘッダRTP・応答不要の読出し系コマンド",
        ),
        0xC860 => Some("MODBUS/TCPによる交信で，CPU応答監視タイマがタイムアウトした。"),
        0xC861 => Some(
            "MODBUS/TCPによる交信で，サポートしていないファンクションコードの要求伝文を受信した。",
        ),
        0xC862 => {
            Some("MODBUS/TCPによる交信で，サポートしていないサブコードの要求伝文を受信した。")
        }
        0xC863 => Some(
            "MODBUS/TCPによる交信で，受信した要求伝文で指定されたMODBUSデバイスに対して，MODBUSデバイス割付パラメータが設定されていない。",
        ),
        0xC864 => Some(
            "MODBUS/TCPによる交信で，受信した要求伝文で指定されたMODBUSデバイスの範囲が，MODBUSデバイス割付パラメータの設定範囲を超えている。",
        ),
        0xC865 => Some(
            "MODBUS/TCPによる交信で，受信した要求伝文で指定されたMODBUSデバイスの範囲が，MODBUSデバイスの上限を超えている。(拡張ファイルレジスタの上限は10000，MODBUSデバイスは65536です。)",
        ),
        0xC866 => Some(
            "MODBUS/TCPによる交信で，受信した要求伝文で指定されたMODBUSデバイスの先頭アドレス，アクセス点数が異常である。",
        ),
        0xC867 => Some(
            "MODBUS/TCPによる交信で，受信した要求伝文の書込み点数指定とバイト数指定が合っていない。",
        ),
        0xC868 => Some(
            "MODBUS/TCPによる交信で，受信した書込みデータサイズと，バイト数指定が合っていない。",
        ),
        0xC869 => Some(
            "MODBUS/TCPによる交信で，受信した要求伝文(FC20，FC21)で指定されたリファレンスタイプの値が異常である。",
        ),
        0xC86A => Some(
            "MODBUS/TCPによる交信で，受信した要求伝文のデータ部の内容が異常である。受信した要求伝文のサイズが，該当ファンクションコードに必要な最低サイズより小さい，または最大サイズより大きい。",
        ),
        0xC86B => Some("MODBUS/TCPによる交信で，受信した要求伝文のMBAPヘッダの内容が異常である。"),
        0xC86C => Some("同時受信可能要求伝文数を越える要求伝文を受信した。"),
        0xCEE0 => {
            Some("接続機器の自動検出中に，他の周辺機器から検出，または他のiQSS機能を実行した。")
        }
        0xCEE1 => Some("異常なフレームを受信した。"),
        0xCEE2 => Some("異常なフレームを受信した。"),
        0xCF10 => Some("異常なフレームを受信した。"),
        0xCF20 => Some(
            "・通信設定の設定値が範囲外である。・対象機器に設定できない通信設定項目を設定した。・対象機器で設定必須の項目が未設定である。",
        ),
        0xCF30 => Some("対象機器がサポートしていないパラメータを指定した。"),
        0xCF31 => Some("異常なフレームを受信した。"),
        0xCF70 => Some("Ethernetの通信経路で異常が発生した。"),
        0xCF71 => Some("タイムアウトエラーが発生した。"),
        0xCF80 => Some("シンプル機器通信で，交信相手と接続できない。または切断された。"),
        0xCF81 => Some("シンプル機器通信で，通信タイムアウトにより送受信に失敗した。"),
        0xCF82 => Some(
            "シンプル機器通信で，送信パケットに設定した”変換なし変数(可変長)”または”データ数可変”の”変換あり変数(可変長)”において，”送受信データ長格納エリア”または”送受信データ数格納エリア”の値が設定可能な範囲を超えている。",
        ),
        0xCF83 => Some("シンプル機器通信で，送信データのデータサイズが0になっている。"),
        0xCF84 => Some("シンプル機器通信で，送信パケットの送信に失敗した。"),
        0xCF85 => Some("シンプル機器通信で，範囲外のデータを設定した。"),
        0xCF8A => Some("シンプル機器通信で，CPUユニットに対する要求に失敗した。"),
        0xCF8C => Some(
            "シンプル機器通信で，受信データがパラメータで設定した全受信パケットと照合一致しなかった。",
        ),
        0xCFB0 => Some("シンプルCPU通信で，再送タイムアウトにより送信に失敗した。"),
        0xCFB1 => Some("シンプルCPU通信で，交信相手と接続できない。または切断された。"),
        0xCFB2 => Some("シンプルCPU通信で，指定された自局ポート番号は重複して使用されている。"),
        0xCFB3 => Some("シンプルCPU通信で，CPUユニットに対する要求に失敗した。"),
        0xCFB4 => Some("シンプルCPU通信で，相手機器から異常応答を受信した。"),
        0xCFB5 => Some("シンプルCPU通信で，相手機器から受信したフレームが異常である。"),
        0xCFBD => Some("シンプルCPU通信で，交信相手側に指定したデバイスが仕様の範囲外である。"),
        0xCFBE => Some("シンプルCPU通信診断実行時にエンジニアリングツールとの交信異常が発生した。"),
        0xCFBF => Some("シンプルCPU通信を実行できない。"),
        0xD000 => Some("ネットワークユニットの異常を検出した。"),
        0xD038 => Some("IP通信テストで指定した対象局が接続されていない。"),
        0xD039 => Some("IP通信テストの経路上に，IPパケット中継機能に未対応である局が存在する。"),
        0xD03B => Some(
            "CPUユニットの動作状態がSTOP(停止エラーを除く)ではないため，リモート機器テスト機能の有効化に失敗した。",
        ),
        0xD041 => Some("通信局数に異常がある。"),
        0xD080 => Some("ネットワークユニットの異常を検出した。"),
        0xD081 => Some("ネットワークユニットの異常を検出した。"),
        0xD082 => Some("ネットワークユニットの異常を検出した。"),
        0xD083 => Some("ネットワークユニットの異常を検出した。"),
        0xD0A0 => Some("トランジェント伝送の送信で応答待ちタイムアウトとなった。"),
        0xD0A1 => Some("トランジェント伝送の送信で送信完了待ちタイムアウトとなった。"),
        0xD0A2 => Some("トランジェント伝送の送信で送信処置待ちタイムアウトとなった。"),
        0xD0A3 => Some("トランジェント伝送の送信処理が正常に行われなかった。"),
        0xD0A4 => Some("トランジェント伝送の送信に失敗した。"),
        0xD0A5 => Some("トランジェント伝送の送信に失敗した。"),
        0xD0A6 => Some("トランジェント伝送の送信に失敗した。"),
        0xD0C0 => Some("予約局指定の処理中に，再度予約局指定を行った。"),
        0xD0C1 => Some("予約局一時解除指定の処理中に，再度予約局一時解除指定を行った。"),
        0xD0C4 => Some("一時エラー無効局設定の処理中に，再度一時エラー無効局設定を行った。"),
        0xD0C5 => Some(
            "一時エラー無効局設定解除指定の処理中に，再度一時エラー無効局設定解除指定を行った。",
        ),
        0xD0D0 => Some("他局の局番設定が正常に行われなかった。"),
        0xD200 => Some("トランジェント伝送で二重に受信した。"),
        0xD202 => Some("送信バッファフルとなった。"),
        0xD203 => Some("トランジェント伝送のデータ読出し，書込みアドレスに誤りがある。"),
        0xD204 => Some("トランジェント伝送のネットワークNo.に誤りがある。"),
        0xD205 => Some("トランジェント伝送の対象局番に誤りがある。"),
        0xD206 => Some("トランジェント伝送のネットワークNo.に誤りがある。"),
        0xD207 => Some(
            "・トランジェント伝送で，他のネットワークへの中継回数が7回を超えた。・二重化システムの待機系からトランジェント伝送を実行した。",
        ),
        0xD208 => Some("トランジェント伝送のネットワークNo.に誤りがある。"),
        0xD209 => Some("トランジェント伝送の対象局番に誤りがある。"),
        0xD20A => Some("トランジェント伝送の対象局番に誤りがある。"),
        0xD20B => Some("トランジェント伝送で指定マスタ局指定時にマスタ局が存在しない。"),
        0xD20C => Some("トランジェント伝送で現在マスタ局指定時にマスタ局が存在しない。"),
        0xD20D => Some("トランジェントデータ伝送の送信で送信完了待ちタイムアウトとなった。"),
        0xD20E => Some("トランジェント伝送のヘッダ情報に誤りがある。"),
        0xD20F => Some(
            "トランジェント伝送の全局指定，またはグループ指定で要求できないコマンドを，全局指定，またはグループ指定で実行した。",
        ),
        0xD210 => Some("トランジェント伝送の対象局番に誤りがある。"),
        0xD211 => Some("自局の局番が未確定のときに，トランジェント伝送を実行した。"),
        0xD212 => Some("トランジェント伝送の送信に失敗した。"),
        0xD213 => Some(
            "・トランジェント伝送のコマンドに誤りがある。・中継受信局の所属するネットワークに対してCC-Link IEフィールドネットワーク診断を使用した。",
        ),
        0xD214 => Some("トランジェント伝送のデータ長に誤りがある。"),
        0xD215 => Some("トランジェント伝送が実行できないユニット動作モードに設定されている。"),
        0xD216 => Some("トランジェント伝送のコマンドに誤りがある。"),
        0xD217 => Some("トランジェント伝送のコマンドに誤りがある。"),
        0xD218 => Some("トランジェント伝送の読出し/書込みデータ数に誤りがある。"),
        0xD219 => Some("トランジェント伝送の属性コードに誤りがある。"),
        0xD21A => Some("トランジェント伝送のアクセスコードに誤りがある。"),
        0xD21B => Some("トランジェント伝送の異常を検出した。"),
        0xD21C => Some("トランジェント伝送の異常を検出した。"),
        0xD21D => Some("トランジェント伝送のネットワークNo.に誤りがある。"),
        0xD21E => Some("トランジェント伝送の対象局番に誤りがある。"),
        0xD21F => Some("専用命令の対象局の局種別指定に誤りがある。"),
        0xD220 => Some("マスタ局が存在しない。"),
        0xD222 => Some("トランジェント伝送のコマンドに誤りがある。"),
        0xD223 => Some("トランジェント伝送の異常を検出した。"),
        0xD224 => Some("トランジェント伝送の異常を検出した。"),
        0xD22E => Some("局番変更対象局に局番設定機能がない。"),
        0xD22F => Some("局番変更対象局がバトンパス未実施である。"),
        0xD230 => Some("局番変更対象局の指定が不正である。"),
        0xD231 => Some("局番変更対象局で局番が設定済みである。"),
        0xD232 => Some("局番変更対象局が存在しない。"),
        0xD233 => Some("局番変更対象局の局番指定に誤りがある。"),
        0xD234 => Some("バトンパスが未実施である。"),
        0xD235 => Some("トランジェント伝送の異常を検出した。"),
        0xD236 => Some("IPデータのTTLに誤りがある。"),
        0xD237 => Some("IPアドレスの設定に誤りがある。"),
        0xD238 => Some("送信キューフルとなった。"),
        0xD239 => Some("SLMP送信に失敗した。"),
        0xD23A => Some("SLMPのサブヘッダに誤りがある。"),
        0xD23B => Some("SLMPのネットワークNo.に誤りがある。"),
        0xD23C => Some("SLMPの対象局番に誤りがある。"),
        0xD23D => Some("デバイス局情報の取得に失敗した。"),
        0xD23E => Some("デバイス局情報の取得に失敗した。"),
        0xD240 => Some("専用命令のネットワークNo.指定に誤りがある。"),
        0xD241 => Some("専用命令の対象局番に誤りがある。"),
        0xD242 => Some("専用命令のコマンドコードに誤りがある。"),
        0xD243 => Some("専用命令のチャンネル指定に誤りがある。"),
        0xD244 => Some("トランジェント伝送データに不正がある。"),
        0xD245 => Some("専用命令の対象局番に誤りがある。"),
        0xD246 => Some("トランジェント伝送データに不正がある。"),
        0xD247 => Some("専用命令で対象局からの応答を二重に受信した。"),
        0xD249 => Some("専用命令の対象局CPU種別に誤りがある。"),
        0xD24A => Some("専用命令の到達監視時間指定に誤りがある。"),
        0xD24B => Some("専用命令の再送回数指定に誤りがある。"),
        0xD24C => Some("専用命令のネットワークNo.指定に誤りがある。"),
        0xD24D => Some("専用命令のチャンネル指定に誤りがある。"),
        0xD24E => Some("専用命令の変更対象指定に誤りがある。"),
        0xD24F => Some("自局の局番が未確定のときに，専用命令を実行した。"),
        0xD251 => Some("専用命令が到達確認異常となった。"),
        0xD252 => Some("専用命令が送信完了待ちタイムアウトとなった。"),
        0xD253 => Some("専用命令が応答タイムアウトとなった。"),
        0xD254 => Some("対象局がサポートしていない専用命令を実行した。"),
        0xD255 => Some("専用命令の対象局番に誤りがある。"),
        0xD256 => Some("専用命令の実行・異常時完了タイプに誤りがある。"),
        0xD257 => Some("REQ命令のリクエストタイプに誤りがある。"),
        0xD258 => {
            Some("専用命令を指定管理局/現在管理局に対して実行したとき，管理局が不在であった。")
        }
        0xD25A => Some("使用されているチャンネルを指定して専用命令を実行した。"),
        0xD25B => Some("使用されているチャンネルを指定して専用命令を実行した。"),
        0xD25C => Some("専用命令の機能バージョン指定に誤りがある。"),
        0xD25D => Some("トランジェント伝送データに不正がある。"),
        0xD25E => Some("同時実行できない専用命令が同時に実行された。"),
        0xD25F => Some("REMFR/REMTO/REMFRD/REMTOD命令を，実行できない局種別のユニットで実行した。"),
        0xD260 => Some("REMTO/REMTOD命令を，実行できない局種別のユニットで実行した。"),
        0xD261 => Some("CCPASET命令を，実行できない局種別のユニットで実行した。"),
        0xD262 => Some("CCPASET命令の総子局数指定に誤りがある。"),
        0xD263 => Some("CCPASET命令のコンスタントリンクスキャンタイム設定に誤りがある。"),
        0xD264 => Some("CCPASET命令の局番設定に誤りがある。"),
        0xD265 => Some("CCPASET命令で設定する局番が重複している。"),
        0xD266 => Some("CCPASET命令のデバイス局設定情報に誤りがある。"),
        0xD267 => Some("CCPASET命令の局種別に誤りがある。"),
        0xD268 => Some("CCPASET命令の各局のリンクデバイスの範囲割付設定に誤りがある。"),
        0xD269 => Some(
            "・REMFR/REMTO/REMFRD/REMTOD命令の対象局の局種別が，インテリジェントデバイス局/リモートデバイス局ではない。・SINFTYRD/SINFSTRD命令の対象局の種別が，インテリジェントデバイス局(リモートヘッドユニット)ではない。",
        ),
        0xD26A => Some("REMFR/REMTO/REMFRD/REMTOD/SINFTYRD/SINFSTRD命令の対象局が存在しない。"),
        0xD26B => Some("CCPASET命令実行局のネットワークNo.設定が異常である。"),
        0xD26C => Some("CCPASET命令実行局の局種別，局番号が異常である。"),
        0xD26F => Some("CCPASET命令のサブマスタ局の局番設定に誤りがある。"),
        0xD270 => Some("CCPASET命令のサブマスタ局設定が複数存在する。"),
        0xD271 => Some("CCPASET命令の予約局設定でサブマスタ局が指定されている。"),
        0xD272 => Some("CCPASET命令のエラー無効局設定でサブマスタ局が指定されている。"),
        0xD273 => Some("トランジェント伝送の要求データサイズに誤りがある。"),
        0xD274 => Some("ルーチング設定に誤りがある。"),
        0xD275 => Some("専用命令が実行中であり，処理が行えない。"),
        0xD276 => Some("専用命令の対象局種別に誤りがある。"),
        0xD277 => Some("ネットワークユニットがサポートしていない専用命令を実行した。"),
        0xD278 => Some("SLMPREQ命令の対象局ネットワークNo.に誤りがある。"),
        0xD279 => Some("ネットワークユニットがサポートしていない専用命令を実行した。"),
        0xD27A => Some("UINI命令で設定した自局局番号に誤りがある。"),
        0xD280 => Some("トランジェント伝送の要求コマンドに誤りがある。"),
        0xD281 => Some("トランジェント伝送の受信に失敗した。"),
        0xD282 => Some("受信キューフルとなった。"),
        0xD283 => Some("トランジェント伝送の送信に失敗した。"),
        0xD284 => Some("SLMPの対象先実行モジュールに誤りがある。"),
        0xD2A0 => Some("受信バッファフルとなった。"),
        0xD2A1 => Some("送信バッファフルとなった。"),
        0xD2A2 => Some("トランジェント伝送で送信完了待ちタイムアウトとなった。"),
        0xD2A3 => Some("トランジェント伝送フレームのデータ長に誤りがある。"),
        0xD2A4 => Some("トランジェント伝送フレームのヘッダ情報に誤りがある。"),
        0xD2A5 => Some("トランジェント伝送フレームの対象局番に誤りがある。"),
        0xD2A6 => Some("トランジェント伝送フレームの要求元番号に誤りがある。"),
        0xD2A7 => Some("トランジェント伝送フレームのヘッダ情報に誤りがある。"),
        0xD2A8 => Some("トランジェント伝送フレームのヘッダ情報に誤りがある。"),
        0xD2A9 => Some("トランジェント伝送フレームの対象ネットワークNo.に誤りがある。"),
        0xD2AA => Some("トランジェント伝送フレームの対象局番に誤りがある。"),
        0xD2AB => Some("トランジェント伝送フレームの要求元ネットワークNo.に誤りがある。"),
        0xD2AC => Some("トランジェント伝送フレームの要求元局番に誤りがある。"),
        0xD2AD => Some("トランジェント伝送フレームのデータ長に誤りがある。"),
        0xD2AE => Some("トランジェント伝送フレームの対象局番に誤りがある。"),
        0xD2AF => Some("トランジェント伝送の対象局番に自局の局番が指定された。"),
        0xD2B0 => Some("トランジェント伝送の送信に失敗した。"),
        0xD2B1 => Some("受信キューフルとなった。"),
        0xD2E0 => Some("IP通信テスト実行中に，再度実行した。"),
        0xD2E1 => Some("IP通信テストが異常完了した。(PING応答なし)"),
        0xD602 => Some("パラメータ異常"),
        0xD605 => Some("パラメータ異常"),
        0xD611 => Some("パラメータ異常(各局デバイス範囲割付異常(RWw))"),
        0xD612 => Some("パラメータ異常(各局デバイス範囲割付異常(RWw))"),
        0xD613 => Some("パラメータ異常(各局デバイス範囲割付異常(RWr))"),
        0xD614 => Some("パラメータ異常(各局デバイス範囲割付異常(RWr))"),
        0xD615 => Some("パラメータ異常(各局デバイス範囲割付異常(RY))"),
        0xD616 => Some("パラメータ異常(各局デバイス範囲割付異常(RY))"),
        0xD617 => Some("パラメータ異常(各局デバイス範囲割付異常(RX))"),
        0xD618 => Some("パラメータ異常(各局デバイス範囲割付異常(RX))"),
        0xD619 => Some("パラメータ異常"),
        0xD61A => Some("パラメータ異常"),
        0xD61B => Some("パラメータ異常(デバイス重複異常(RWw))"),
        0xD61C => Some("パラメータ異常(デバイス重複異常(RWr))"),
        0xD61D => Some("パラメータ異常（デバイス重複異常(RY)）"),
        0xD61E => Some("パラメータ異常(デバイス重複異常(RX))"),
        0xD61F => Some("CCPASET命令によるパラメータ設定が正常に行われなかった。"),
        0xD620 => Some("トランジェント伝送データに不正がある。"),
        0xD621 => Some("パラメータ異常"),
        0xD622 => Some("パラメータ異常(総子局数異常)"),
        0xD623 => Some("パラメータ異常(リンクスキャンモード異常)"),
        0xD624 => Some("パラメータ異常(コンスタントリンクスキャンタイム設定異常)"),
        0xD625 => Some("パラメータ異常(局単位ブロック保証設定異常)"),
        0xD626 => Some("パラメータ異常(ループバック設定異常)"),
        0xD628 => Some("パラメータ異常(局種別異常)"),
        0xD629 => Some("パラメータ異常(局番号範囲異常)"),
        0xD62A => Some("パラメータ異常(データリンク異常局設定異常)"),
        0xD62B => Some("パラメータ異常(CPU STOP時の出力設定異常)"),
        0xD630 => Some("ローカル局でのCCPASET命令によるパラメータ設定が正常に行われなかった。"),
        0xD634 => Some("パラメータ異常(サブマスタ設定数異常)"),
        0xD635 => Some("パラメータ異常(サブマスタ局番異常)"),
        0xD636 => Some("ローカル局以外でUINI命令を実行した。"),
        0xD637 => Some(
            "・パラメータで局番が設定されている局でUINI命令を実行した。・二重化システムでUINI命令を実行した。",
        ),
        0xD638 => Some("UINI命令で設定した自局の局番が他局と重複している。"),
        0xD639 => Some("UINI命令で局番を設定後，再度UINI命令を実行した。"),
        0xD63D => Some("パラメータ異常"),
        0xD63E => Some("パラメータ異常"),
        0xD641 => Some("パラメータ異常(IPアドレス異常)"),
        0xD701 => {
            Some("対象局を指定せず，予約局一時解除/取消，一時エラー無効局設定/取消を要求した。")
        }
        0xD706 => Some("ローカル局から予約局一時解除/取消，一時エラー無効局設定/取消を要求した。"),
        0xD70B => Some("予約局一時解除/取消，一時エラー無効局設定/取消を同時に要求した。"),
        0xD720 => Some("リンク起動/停止の指示内容に誤りがある。"),
        0xD721 => Some("リンク起動/停止の処理中に，他局からリンク起動/停止が要求された。"),
        0xD722 => Some("リンク起動/停止の処理中に，自局からリンク起動/停止を要求した。"),
        0xD723 => Some("リンク起動/停止の処理中に，システムのリンク起動/停止が要求された。"),
        0xD724 => Some("リンク起動/停止の局指定に誤りがある。"),
        0xD725 => Some("ローカル局からシステムのリンク起動/停止を要求した。"),
        0xD726 => Some("トランジェント伝送の要求コマンドに誤りがある。"),
        0xD727 => Some("リンク起動を，リンク停止を要求した局以外から要求した。"),
        0xD728 => Some("データリンク中の局に対してデータリンク起動指示を行った。"),
        0xD729 => Some("局番未設定の局で，自局のリンク停止を指示した。"),
        0xD731 => Some("強制マスタ切替え指示を，マスタ動作中のサブマスタ局以外から行った。"),
        0xD740 => Some("トランジェント伝送の送信に失敗した。"),
        0xD741 => Some("実行局の局種別異常を検出した。"),
        0xD742 => Some("トランジェント伝送の送信に失敗した。"),
        0xD743 => Some("実行局の局種別異常を検出した。"),
        0xD744 => Some("フラッシュROMのクリアに失敗した。"),
        0xD783 => Some("トランジェント伝送の異常を検出した。"),
        0xD784 => Some("トランジェント伝送の異常を検出した。"),
        0xD806 => Some("受信キューフルとなった。"),
        0xD840 => Some("トランジェントの要求数が，配送処理で同時処理可能な上限を超過した。"),
        0xD841 => Some("メモリ読書きコマンドの要求データサイズが範囲外である。"),
        0xD842 => Some(
            "・宛先ネットワークNo.へのルーチング情報が未登録である。・トランジェント伝送で，他のネットワークへの中継回数が7回を超えた。",
        ),
        0xD843 => Some("トランジェント伝送が実行できないユニット動作モードに設定されている。"),
        0xD844 => Some(
            "異常なフレームを受信した。・未対応変換前プロトコル・未対応フレームタイプ・アプリケーションヘッダ可変部・アプリケーションヘッダHDS・アプリケーションヘッダRTP・応答不要の読出し系コマンド",
        ),
        0xD902 => Some("オンラインテストデータに不正がある。"),
        0xD903 => Some("交信テスト実行中に，再度実行した。"),
        0xD905 => Some("交信テストが交信監視時間タイムアウトとなった。"),
        0xD906 => Some("交信テストが送信完了待ちタイムアウトとなった。"),
        0xD909 => Some("トランジェント伝送のヘッダ情報に誤りがある。"),
        0xD90A => Some("交信テスト実行中に，再度実行した。"),
        0xD90B => Some("ネットワーク内で通信している局数が仕様の範囲外である。"),
        0xD90C => Some("交信テストの対象局の指定に誤りがある。"),
        0xD90D => Some("ケーブルテスト実行中に，再度実行した。"),
        0xD90E => Some("IPパケット中継機能に対応していない。"),
        0xD90F => Some("IP通信テスト実行中に処理を行った。"),
        0xD910 => Some("自局のIPアドレスが設定されていない。"),
        0xD911 => Some("IP通信テストの宛先のIPアドレス設定に異常がある。"),
        0xD912 => Some("トランジェント伝送の送信に失敗した。"),
        0xD913 => Some("ネットワークユニットの異常を検出した。"),
        0xD914 => Some("ネットワークユニットの異常を検出した。"),
        0xD915 => Some("ネットワークユニットの異常を検出した。"),
        0xD916 => Some("ネットワークユニットの異常を検出した。"),
        0xD917 => Some("ネットワークユニットの異常を検出した。"),
        0xD918 => Some(
            "IP通信テストの”接続局(自局)”，または”通信先設定”が待機系のIPアドレスになっている。",
        ),
        0xDA00 => Some("ネットワークユニットの異常を検出した。"),
        0xDA01 => Some("ネットワークユニットの異常を検出した。"),
        0xDA10 => Some("ネットワークユニットの異常を検出した。"),
        0xDA11 => Some("ネットワークユニットの異常を検出した。"),
        0xDA12 => Some("ネットワークユニットの異常を検出した。"),
        0xDA13 => Some("ネットワークユニットの異常を検出した。"),
        0xDA14 => Some("ネットワークユニットの異常を検出した。"),
        0xDA15 => Some("ネットワークユニットの異常を検出した。"),
        0xDA16 => Some("ネットワークユニットの異常を検出した。"),
        0xDA17 => Some("ネットワークユニットの異常を検出した。"),
        0xDA19 => Some("ネットワークユニットの異常を検出した。"),
        0xE006 => Some("受信処理用のキューの最大数を使用している。"),
        0xE102 => Some("自局が，予約局に設定されている。"),
        0xE103 => Some("自局が，総局数の範囲外に設定されている。"),
        0xE120 => Some("管理局でUINI命令を実行した。"),
        0xE121 => Some(
            "・”必須設定”の”局番設定”の”局番設定方法”が”パラメータで設定”の状態でUINI命令を実行した。・二重化システムでUINI命令を実行した。",
        ),
        0xE122 => Some("UINI命令で設定した自局の局番が他局と重複している。"),
        0xE123 => Some("UINI命令で局番を設定後，再度UINI命令を実行した。"),
        0xE160 => Some("‘リンク起動/停止の指示内容’(SW0000)が正しく設定されていない。"),
        0xE162 => {
            Some("サイクリック伝送の停止または再開の処理中に，リンク起動/停止を再度実行した。")
        }
        0xE163 => {
            Some("サイクリック伝送の停止または再開の処理中に，リンク起動/停止を再度実行した。")
        }
        0xE164 => {
            Some("サイクリック伝送の停止または再開の処理中に，リンク起動/停止を再度実行した。")
        }
        0xE165 => Some("‘リンク起動/停止の局指定’(SW0001~SW0008)が，正しく設定されていない。"),
        0xE166 => {
            Some("‘リンク起動/停止のグループ指定’(SW0012~SW0013)が，正しく設定されていない。")
        }
        0xE170 => Some("ネットワークユニットの異常を検出した。"),
        0xE171 => Some("ネットワークユニットの異常を検出した。"),
        0xE172 => Some("ネットワークユニットの異常を検出した。"),
        0xE173 => Some("交信テスト実行中に，再度実行した。"),
        0xE174 => Some("送信完了リトライアウトが発生した。"),
        0xE175 => Some("交信監視時間までに応答が返らなかった。"),
        0xE176 => Some("送信完了せずにタイムアウトした。"),
        0xE177 => Some("ネットワークユニットの異常を検出した。"),
        0xE178 => Some("ネットワークユニットの異常を検出した。"),
        0xE179 => Some("ネットワークユニットの異常を検出した。"),
        0xE17A => Some("応答データを2回以上受信した。"),
        0xE17B => Some("ネットワークユニットの異常を検出した。"),
        0xE17C => Some("交信テストの対象局の指定に誤りがある。"),
        0xE17D => Some("IP通信テスト時に自局のIPアドレスが取得できない。"),
        0xE17E => Some(
            "IP通信テストの通信先設定に設定されたIPアドレスの第1オクテットと第2オクテットが，要求元機器から要求先機器までのネットワークで統一されていない。",
        ),
        0xE17F => Some("ネットワークユニットの異常を検出した。"),
        0xE180 => Some("ケーブルテスト実行中に，再度実行した。(Ethernetケーブル使用時のみ)"),
        0xE181 => Some("IPパケット中継機能に対応していない。"),
        0xE182 => Some("IP通信テスト実行中に，再度実行した。"),
        0xE183 => Some("トランジェント伝送の送信に失敗した。"),
        0xE184 => Some("ネットワークユニットの異常を検出した。"),
        0xE185 => Some("ネットワークユニットの異常を検出した。"),
        0xE186 => Some(
            "IP通信テストの”接続局(自局)”，または”通信先設定”が待機系のIPアドレスになっている。",
        ),
        0xE201 => Some("同一のトランジェントデータを2回以上受信した。"),
        0xE203 => Some("送信バッファが，最大数使用されている。"),
        0xE204 => Some("指定された回数分，再送処理を実行した。"),
        0xE205 => Some("受信バッファが，最大数使用されている。"),
        0xE206 => Some("ネットワークユニットの異常を検出した。"),
        0xE207 => Some(
            "トランジェント伝送の対象局が，同一ネットワーク内に接続されているにもかかわらず，別のネットワークNo.が設定されている。",
        ),
        0xE208 => Some("トランジェント伝送で指定された対象局番が範囲外である。"),
        0xE20A => Some("ネットワークユニットの異常を検出した。"),
        0xE20B => Some("トランジェント伝送で，他のネットワークへの中継回数が7回を超えた。"),
        0xE20F => Some("SLMPなどによるトランジェント伝送で，対象局番を0に指定した。"),
        0xE211 => Some(
            "SLMPなどによるトランジェント伝送で，対象局を”指定管理局”に指定した場合に，管理局が存在しない。",
        ),
        0xE212 => Some(
            "SLMPなどによるトランジェント伝送で，対象局を”現在管理局”に指定した場合に，管理局が存在しない。",
        ),
        0xE213 => Some("トランジェント伝送の送信時，送信完了待ちがタイムアウトした。"),
        0xE215 => Some("ネットワークユニットの異常を検出した。"),
        0xE216 => Some("ネットワークユニットの異常を検出した。"),
        0xE218 => Some("ネットワークユニットの異常を検出した。"),
        0xE21B => Some("自局の局番が未確定のときに，トランジェント伝送を実行した。"),
        0xE21C => Some("ネットワークユニットの異常を検出した。"),
        0xE21E => Some("ネットワークユニットの異常を検出した。"),
        0xE21F => Some("ネットワークユニットの異常を検出した。"),
        0xE221 => Some("ネットワークユニットの異常を検出した。"),
        0xE222 => Some("ネットワークユニットの異常を検出した。"),
        0xE223 => Some("ネットワークユニットの異常を検出した。"),
        0xE224 => Some("CC-Linkトランジェント要求フレーム内の属性コードが範囲外である。"),
        0xE225 => Some("CC-Linkトランジェント要求フレーム内のアクセスコードが範囲外である。"),
        0xE226 => Some("ネットワークユニットの異常を検出した。"),
        0xE228 => Some(
            "・トランジェント伝送の要求コマンドに誤りがある。・中継受信局の所属するネットワークに対してCC-Link IEコントローラネットワーク診断を使用した。",
        ),
        0xE229 => Some("管理局が存在しない。"),
        0xE22A => Some("トランジェント伝送の異常を検出した。"),
        0xE22B => Some("バトンパスが未実施である。"),
        0xE22C => Some("トランジェント伝送の異常を検出した。"),
        0xE22D => Some("トランジェント伝送の異常を検出した。"),
        0xE236 => Some("IPデータのTTLに誤りがある。"),
        0xE237 => Some("IPアドレス設定に誤りがある。"),
        0xE241 => Some("専用命令の対象局のネットワークユニットのハードウェアが故障した。"),
        0xE242 => Some("専用命令の対象局のネットワークユニットのハードウェアが故障した。"),
        0xE243 => Some("専用命令の対象局のネットワークユニットのハードウェアが故障した。"),
        0xE244 => Some("専用命令の対象局のネットワークユニットのハードウェアが故障した。"),
        0xE245 => Some("専用命令の対象局のネットワークユニットのハードウェアが故障した。"),
        0xE24F => Some("専用命令実行時，設定された局番が，仕様の範囲外である。"),
        0xE251 => Some("同一の専用命令用のトランジェントデータを2回以上受信した。"),
        0xE254 => Some("専用命令実行時，設定された対象局CPU種別が，仕様の範囲外である。"),
        0xE255 => Some("専用命令実行時，設定されたデータサイズが，仕様の範囲外である。"),
        0xE256 => Some("専用命令実行時，設定された到達監視時間が，仕様の範囲外である。"),
        0xE257 => Some("専用命令実行時，設定された再送回数が，仕様の範囲外である。"),
        0xE258 => Some("専用命令実行時，設定されたネットワークNo.が，仕様の範囲外である。"),
        0xE259 => Some("専用命令の使用チャンネルに誤りがある。"),
        0xE25A => Some("UINI命令実行時，設定された変更対象指定が，仕様の範囲外である。"),
        0xE25B => Some("専用命令実行時，設定された自局局番が，仕様の範囲外である。"),
        0xE262 => Some(
            "専用命令実行時，対象局がグループ指定，または，全局指定時に，実行タイプが到達確認ありに設定された。REQ命令の場合は，指定されたリクエストタイプが誤っている。",
        ),
        0xE264 => Some("専用命令実行後，送信完了が実施されず，タイムアウトした。"),
        0xE265 => Some("専用命令実行後，応答受信が実施されず，タイムアウトした。"),
        0xE266 => Some("他ネットワークからのSEND命令を受信した。"),
        0xE267 => Some("対象局番に自局の局番が指定された。"),
        0xE268 => Some("実行・異常時完了タイプ指定で，0固定のエリアのbitがONしている。"),
        0xE269 => Some("REQ命令のリクエストタイプ・サブリクエストタイプの指定に誤りがある。"),
        0xE26A => Some(
            "ネットワーク上に管理局が存在しないときに，指定管理局を指定，または現在管理局を指定して，専用命令を実行した。",
        ),
        0xE26C => Some("使用されているチャンネルを使用した。"),
        0xE26D => Some("イベントパラメータにて設定されているチャンネルを使用した。"),
        0xE26E => Some("ZNRD/ZNWR命令にて使用したデバイス指定が誤っている。"),
        0xE26F => Some("ZNRD/ZNWR命令にて使用したデバイス指定が誤っている。"),
        0xE271 => Some("REQ命令(リモートRUN/STOP)の動作モードの指定に誤りがある。"),
        0xE272 => {
            Some("REQ命令(リモートRUN/STOP)でリモートRUNを指定時，クリアモードの指定に誤りがある。")
        }
        0xE273 => Some("RRUN命令にて，誤ったコントロールデータが指定されている。"),
        0xE274 => Some("ネットワークユニットの異常を検出した。"),
        0xE277 => Some("ネットワークユニットの異常を検出した。"),
        0xE278 => Some("トランジェント伝送の要求データサイズが仕様の範囲外である。"),
        0xE279 => Some("ルーチング設定に誤りがある。"),
        0xE27A => Some("同時実行できない専用命令が同時に実行された。"),
        0xE27B => Some("専用命令の対象局種別の指定に誤りがある。"),
        0xE27C => Some("ネットワークユニットの異常を検出した。"),
        0xE27D => Some("ネットワークユニットの異常を検出した。"),
        0xE286 => Some("ネットワークユニットの異常を検出した。"),
        0xE2A0 => Some("CC-Link専用命令用の受信バッファに空きが無い。"),
        0xE2A1 => Some("CC-Link専用命令用の送信バッファに空きが無い。"),
        0xE2A2 => Some("ネットワークユニットのハードウェアが故障した。"),
        0xE2A3 => Some("トランジェント伝送フレームのフレーム長(L)に誤りがある。"),
        0xE2A4 => Some("トランジェント伝送のゲートカウント(GCNT)に誤りがある。"),
        0xE2A5 => Some("トランジェント伝送の対象先局No.(DA)に誤りがある。"),
        0xE2A6 => Some("トランジェント伝送フレームの起動元局No.(SA)に誤りがある。"),
        0xE2A7 => {
            Some("トランジェント伝送フレームの対象先アプリケーションタイプ(DAT)に異常がある。")
        }
        0xE2A8 => {
            Some("トランジェント伝送フレームの起動元アプリケーションタイプ(SAT)に異常がある。")
        }
        0xE2A9 => Some("トランジェント伝送フレームの対象先ネットワークNo.(DNA)に誤りがある。"),
        0xE2AA => Some("トランジェント伝送フレームの対象先局No.(DS)に誤りがある。"),
        0xE2AB => Some("トランジェント伝送フレームの起動元ネットワーク№(SNA)に誤りがある。"),
        0xE2AC => Some("トランジェント伝送フレームの起動元局No.(SS)に誤りがある。"),
        0xE2AD => Some("トランジェント伝送フレームのデータ長(L1)に誤りがある。"),
        0xE2AE => Some(
            "トランジェント伝送フレームの対象先局No.(DA)が自局と一致するが，対象先ネットワークNo.(DNA)/対象先局No.(DS)が自局と異なるデータを受信した。",
        ),
        0xE2AF => Some("CC-Link専用命令の対象局番に自局の局番が指定された。"),
        0xE2B0 => Some("ネットワークユニットの異常を検出した。"),
        0xE501 => Some("ネットワークユニットの異常を検出した。"),
        0xE502 => Some("ネットワークユニットの異常を検出した。"),
        0xE503 => Some("ネットワークユニットの異常を検出した。"),
        0xE504 => Some(
            "自局がバトンパス未実施中に，トランジェント伝送(専用命令，エンジニアリングツール接続)を実行した。",
        ),
        0xE505 => Some(
            "自局が局番重複異常中に，トランジェント伝送(専用命令，エンジニアリングツール接続)を実行した。",
        ),
        0xE521 => Some("ネットワークユニットの異常を検出した。"),
        0xE5F0 => Some(
            "対象局がバトンパス未実施中に，トランジェント伝送(専用命令，エンジニアリングツール接続)を実行した。",
        ),
        0xE5F1 => Some("トランジェント伝送の対象局の局番が重複している。"),
        0xE5F8 => Some(
            "IPパケット中継機能による通信時，経路上にIPパケット中継機能に未対応である局が存在する。",
        ),
        0xE840 => Some("トランジェントの要求数が，配送処理で同時処理可能な上限を超過した。"),
        0xE841 => Some("メモリ読書きコマンドの要求データサイズが範囲外である。"),
        0xE842 => Some(
            "・宛先ネットワークNo.へのルーチング情報が未登録である。・トランジェント伝送で，他のネットワークへの中継回数が7回を超えた。",
        ),
        0xE843 => Some("トランジェント伝送が実行できないユニット動作モードに設定されている。"),
        0xE844 => Some(
            "異常なフレームを受信した。・未対応変換前プロトコル・未対応フレームタイプ・アプリケーションヘッダ可変部・アプリケーションヘッダHDS・アプリケーションヘッダRTP・応答不要の読出し系コマンド",
        ),
        0xEA00 => Some("ネットワークユニットの異常を検出した。"),
        0xEA01 => Some("ネットワークユニットの異常を検出した。"),
        _ => None,
    }
}

/// Return the error detail/cause message for an SLMP end code.
pub fn end_code_message(end_code: u16, language: SlmpEndCodeLanguage) -> Option<&'static str> {
    match language {
        SlmpEndCodeLanguage::English => end_code_message_en(end_code),
        SlmpEndCodeLanguage::Japanese => end_code_message_ja(end_code),
    }
}

/// Return whether the SLMP end code is related to remote password protection.
pub fn is_remote_password_end_code(end_code: u16) -> bool {
    matches!(
        end_code,
        0xC200
            | 0xC201
            | 0xC202
            | 0xC203
            | 0xC204
            | 0xC205
            | 0xC810
            | 0xC811
            | 0xC812
            | 0xC813
            | 0xC814
            | 0xC815
            | 0xC816
    )
}
