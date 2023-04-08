use std::ffi::CStr;

use bevy_math::{dvec3, DVec3};
use derive_more::{Deref, Display};

pub mod ffi;
pub mod states;
use ffi::{ScanProfile, Walabot_GetArenaR, Walabot_GetErrorString, Walabot_SetSettingsFolder};

use libloading::{self, Library};
use states::Unconnected;
use uom::si::f64::{Angle, Length};

use crate::ffi::{Walabot_GetArenaTheta, Walabot_GetArenaPhi};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Internal walabot error: {0}")]
    WalabotError(String),
    #[error("Shared library error: {0}")]
    SharedLib(#[from] libloading::Error),
}

impl Error {
    pub fn from_walabot() -> Self {
        let err = unsafe { Walabot_GetErrorString() };
        let s = unsafe { CStr::from_ptr(err) };
        Error::WalabotError(s.to_string_lossy().to_string())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Arena {
    Polar { r: DVec3, theta: DVec3, phi: DVec3 },
    Cartesian { x: DVec3, y: DVec3, z: DVec3 },
}

impl Arena {
    pub fn from_walabot() -> Self {
        fn bail(var: &str) -> Arena {
            eprintln!("Failed to get arena {}: {}", var, Error::from_walabot());
            return Arena::default();
        }
        let mut r = dvec3(0.0, 0.0, 0.0);
        let mut theta = dvec3(0.0, 0.0, 0.0);
        let mut phi = dvec3(0.0, 0.0, 0.0);
        if unsafe { Walabot_GetArenaR(&mut r.x, &mut r.y, &mut r.z) } != 0 {
            return bail("r");
        }
        if unsafe { Walabot_GetArenaTheta(&mut theta.x, &mut theta.y, &mut theta.z) } != 0 {
            return bail("theta");
        }
        if unsafe { Walabot_GetArenaPhi(&mut phi.x, &mut phi.y, &mut phi.z) } != 0 {
            return bail("phi");
        }
        Self::Polar { r, theta, phi }
    }
}

impl Default for Arena {
    fn default() -> Self {
        Arena::Polar {
            r: dvec3(1.0, 10.0, 0.5),
            theta: dvec3(-45.0, 45.0, 1.0),
            phi: dvec3(-45.0, 45.0, 1.0),
        }
    }
}

/// A Python module implemented in Rust.
// #[pymodule]
// fn walabot(_py: Python, m: &PyModule) -> PyResult<()> {
// }

pub fn walabot() -> Result<states::Unconnected> {
    let path = "./db".to_string();
    let out = unsafe { Walabot_SetSettingsFolder(path.as_ptr()) };
    if out != 0 {
        return Err(Error::WalabotError(
            "Failed to set settings folder".to_string(),
        ));
    }
    Ok(Unconnected)
}

#[cfg(test)]
mod tests {
    use crate::states::Disconnect;

    use super::*;

    #[test]
    fn test_connect() {
        let walabot = walabot().expect("Failed to initialize Walabot");
        let mut connected = walabot.connect().expect("Failed to connect to Walabot");
        let _ = dbg!(connected.status());
        connected
            .set_profile(ScanProfile::LongRange)
            .expect("Failed to set profile");
        connected
            .set_arena(Arena::default())
            .expect("Failed to set arena");
        while let Some(image) = connected.start().unwrap().next() {
            match image {
                Ok(image) => {
                    println!("Got image: {:?}", image)
                }
                Err(e) => {
                    let _ = dbg!(e);
                }
            }
        }
        connected.disconnect();
    }
}
