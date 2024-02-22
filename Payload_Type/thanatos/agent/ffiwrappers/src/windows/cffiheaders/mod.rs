pub mod ntdll {
    #![allow(unused, non_snake_case, non_camel_case_types, non_upper_case_globals)]
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/ntdll.rs"));
}
