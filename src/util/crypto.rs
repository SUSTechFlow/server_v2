pub const BCRYPT_COST: u32 = 12;

// 偿还技术债务
// TODO： 统一密码加密方式
pub fn verify_helper(token: &str, password: &str) -> bool {
    let hash = String::from_utf8(base64::decode(&token.replace("\n", "")).unwrap_or(vec![])).unwrap_or(String::new());
    if let Ok(result) = bcrypt::verify(&password, &hash) {
        return result;
    }
    if let Ok(result) = bcrypt::verify(&password, &token) {
        return result;
    }
    return false;
}