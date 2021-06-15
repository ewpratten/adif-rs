/// All possible ADIF awards
///
/// See: https://www.adif.org/312/ADIF_312.htm#Award_Enumeration
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Award {
    AJA,
    CQDX,
    CQDXFIELD,
    CQWAZ_MIXED,
    CQWAZ_CW,
    CQWAZ_PHONE,
    CQWAZ_RTTY,
    CQWAZ_160m,
    CQWPX,
    DARC_DOK,
    DXCC,
    DXCC_MIXED,
    DXCC_CW,
    DXCC_PHONE,
    DXCC_RTTY,
    IOTA,
    JCC,
    JCG,
    MARATHON,
    RDA,
    WAB,
    WAC,
    WAE,
    WAIP,
    WAJA,
    WAS,
    WAZ,
    USACA,
    VUCC,
}
