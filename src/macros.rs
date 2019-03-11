#[macro_export(local_inner_macros)]
macro_rules! register {
    {$($name:expr => $tp:ty),+} => {
        {
            let mut _temp_custom: ::std::collections::HashMap<String, utils::FromJsonFunc> =
                ::std::collections::HashMap::new();
            $(
                _temp_custom.insert(
                    String::from($name),
                    utils::new_from_json::<$tp>
                );
            )+
            _temp_custom
        }
    };
    {} => {
        ::std::collections::HashMap::new()
    };
}

#[macro_export(local_inner_macros)]
macro_rules! vct {
    ($x:expr, $y:expr, $z:expr) => {
        Vct::new($x as utils::Flt, $y as utils::Flt, $z as utils::Flt)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! ray {
    ($x:expr, $y:expr) => {
        Ray::new($x, $y)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! geo {
    ($x:expr, $y:expr, $z:expr) => {
        Geo::new($x, $y, $z)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! sphere {
    ($x:expr, $y:expr, $z:expr) => {
        Sphere::new($x, $y as utils::Flt, $z)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! plane {
    ($x:expr, $y:expr, $z:expr) => {
        Plane::new($x, $y, $z)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! cam {
    ($x:expr, $y:expr, $z:expr) => {
        Cam::new($x, $y, $z as utils::Flt)
    };
}
