use errors::ThanatosError;
use ffiwrappers::linux::user::UserInfo;

pub fn username() -> Result<String, ThanatosError> {
    UserInfo::current_user()
        .map(|user| user.username().to_string())
        .map_err(ThanatosError::FFIError)
}
