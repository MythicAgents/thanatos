use windows::core::HRESULT;

#[derive(Debug)]
pub enum CryptoError {
    Rsa(RsaError),
    Rng(RngError),
    Aes(AesError),
    Base64(Base64Error),

    #[allow(dead_code)]
    Sha256(Sha256Error),

    Hmac(HmacError),
    InvalidData,
}

#[derive(Debug)]
pub enum Base64Error {
    MalformedData,
    DataTooLong,
}

#[derive(Debug)]
pub enum RngError {
    #[cfg(windows)]
    WinError(HRESULT),
}

#[derive(Debug)]
pub enum RsaError {
    #[cfg(windows)]
    WinError(HRESULT),
}

#[derive(Debug)]
pub enum AesError {
    #[cfg(windows)]
    WinError(HRESULT),
}

#[derive(Debug)]
pub enum Sha256Error {
    #[allow(dead_code)]
    #[cfg(windows)]
    WinError(HRESULT),
}

#[derive(Debug)]
pub enum HmacError {
    MacMismatch,
    #[cfg(windows)]
    WinError(HRESULT),
}
