
use once_cell::sync::Lazy;
use regex::Regex;

pub static COMMENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A#.*").expect("regex"));


// TODO consolidate floating points
static PointFloatStr: &str = r#"([0-9](?:_?[0-9])*\\.(?:[0-9](?:_?[0-9])*)?|\\.[0-9](?:_?[0-9])*)([eE][-+]?[0-9](?:_?[0-9])*)?"#;
static PointFloatsStr2: &str = r"[0-9](?:_?[0-9])*\.(?:[0-9](?:_?[0-9])*)?([eE][-+]?[0-9](?:_?[0-9])*)?";
static PointFloatStr3: &str = r"\.[0-9](?:_?[0-9])*([eE][-+]?[0-9](?:_?[0-9])*)?";

static POINTFLOAT: Lazy<Regex> = Lazy::new(|| Regex::new(PointFloatStr).expect("regex"));
static POINTFLOAT1: Lazy<Regex> = Lazy::new(|| Regex::new(PointFloatsStr2).expect("regex"));
static POINTFLOAT2: Lazy<Regex> = Lazy::new(|| Regex::new(PointFloatStr3).expect("regex"));

pub static FLOATING_POINT: Lazy<Regex> = Lazy::new(|| Regex::new(format!(r"\A({}|{}|{})", PointFloatStr, PointFloatsStr2, PointFloatStr3).as_str()).expect("regex"));

static POSSIBLE_NAME_STR: &str = r"[a-zA-Z]{1}[\w\d]+";
static POSSIBLE_NAME_ONE_CHAR: &str = r"[a-zA-Z]{1}";

pub static POSSIBLE_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A[a-zA-Z]{1}[\w\d]+").expect("regex"));
pub static POSSIBLE_ONE_CHAR_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"\A[a-zA-Z]{1}").expect("regex"));

pub static NAME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(format!(r"\A({}|{})", POSSIBLE_NAME_STR, POSSIBLE_NAME_ONE_CHAR ).as_str()).expect("regex"));

