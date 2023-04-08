use std::ffi::c_double;
use std::ffi::c_int;
use std::slice;

use bevy_math::dvec3;

use crate::ffi::Status;
use crate::ffi::Walabot_ConnectAny;
use crate::ffi::Walabot_Disconnect;
use crate::ffi::Walabot_GetArenaPhi;
use crate::ffi::Walabot_GetRawImage;
use crate::ffi::Walabot_GetStatus;
use crate::ffi::Walabot_SetArenaPhi;
use crate::ffi::Walabot_SetArenaR;
use crate::ffi::Walabot_SetArenaTheta;
use crate::ffi::Walabot_SetArenaX;
use crate::ffi::Walabot_SetArenaY;
use crate::ffi::Walabot_SetArenaZ;
use crate::ffi::Walabot_SetProfile;
use crate::ffi::Walabot_Start;
use crate::ffi::Walabot_Stop;
use crate::ffi::Walabot_Trigger;
use crate::Arena;
use crate::Error;

use super::Result;
use super::ScanProfile;

/// Walabot is not connected
#[derive(Debug)]
pub struct Unconnected;

/// Walabot is connected
#[derive(Debug)]
pub struct Connected;

impl Unconnected {
    pub fn connect(self) -> Result<Connected> {
        if unsafe { Walabot_ConnectAny() } != 0 {
            Err(Error::from_walabot())
        } else {
            Ok(Connected)
        }
    }
}

impl Connected {
    pub fn status(&self) -> Result<Status> {
        let mut param = 0.0;
        let mut status = Status::Clean;
        if unsafe { Walabot_GetStatus(&mut status, &mut param) } != 0 {
            Err(Error::from_walabot())
        } else {
            Status::from(status).and(param)
        }
    }

    pub fn set_profile(&mut self, profile: ScanProfile) -> Result<()> {
        if unsafe { Walabot_SetProfile(profile) } != 0 {
            Err(Error::from_walabot())
        } else {
            Ok(())
        }
    }

    pub fn set_arena(&mut self, arena: super::Arena) -> Result<()> {
        fn bail(msg: &str) -> Result<()> {
            Err(Error::WalabotError(format!("Failed to set arena {}", msg)))
        }
        match arena {
            Arena::Polar { r, theta, phi } => {
                let (r_min, r_max, r_res) = (r.x, r.y, r.z);
                let (theta_min, theta_max, theta_res) = (theta.x, theta.y, theta.z);
                let (phi_min, phi_max, phi_res) = (phi.x, phi.y, phi.z);
                if unsafe { Walabot_SetArenaR(r_min, r_max, r_res) } != 0 {
                    return bail("R");
                }
                if unsafe { Walabot_SetArenaTheta(theta_min, theta_max, theta_res) } != 0 {
                    return bail("Theta");
                }
                if unsafe { Walabot_SetArenaPhi(phi_min, phi_max, phi_res) } != 0 {
                    return bail("Phi");
                }
            }
            Arena::Cartesian { x, y, z } => {
                let (x_min, x_max, x_res) = (x.x, x.y, x.z);
                let (y_min, y_max, y_res) = (y.x, y.y, y.z);
                let (z_min, z_max, z_res) = (z.x, z.y, z.z);
                if unsafe { Walabot_SetArenaX(x_min, x_max, x_res) } != 0 {
                    return bail("X");
                }
                if unsafe { Walabot_SetArenaY(y_min, y_max, y_res) } != 0 {
                    return bail("Y");
                }
                if unsafe { Walabot_SetArenaZ(z_min, z_max, z_res) } != 0 {
                    return bail("Z");
                }
            }
        }
        Ok(())
    }

    pub fn start(&mut self) -> Result<Started> { if unsafe { Walabot_Start() } != 0 {
            Err(Error::from_walabot())
        } else {
            Ok(Started {
                arena: Arena::from_walabot(),
            })
        }
    }
}

pub struct Started {
    arena: Arena,
}

impl Iterator for Started {
    type Item = Result<&'static [i32]>;

    fn next(&mut self) -> Option<Self::Item> {
        if unsafe { Walabot_Trigger() } != 0 {
            Some(Err(Error::from_walabot()))
        } else {
            let mut sizeX: c_int = 0;
            let mut sizeY: c_int = 0;
            let mut sizeZ: c_int = 0;
            let mut power: c_double = 0.0;
            let mut rasterImage = std::ptr::null();
            if unsafe {
                Walabot_GetRawImage(&mut rasterImage, &mut sizeX, &mut sizeY, &mut sizeZ, &mut power)
            } != 0
            {
                return Some(Err(Error::from_walabot()));
            }
            let capacity = (sizeX * sizeY * sizeZ) as usize;
            let data = unsafe { slice::from_raw_parts(rasterImage, capacity) };
            println!("sizeX: {}, sizeY: {}, sizeZ: {}, power: {}", sizeX, sizeY, sizeZ, power);
            Some(Ok(data))
        }
    }
}

impl Drop for Started {
    fn drop(&mut self) {
        if unsafe { Walabot_Stop() } != 0 {
            eprintln!("Failed to stop Walabot: {}", Error::from_walabot())
        }
    }
}

pub trait Disconnect: Sized {
    fn disconnect(self) -> Unconnected {
        if unsafe { Walabot_Disconnect() } != 0 {
            eprintln!("Failed to disconnect from Walabot, ignoring");
        }
        Unconnected
    }
}

impl Disconnect for Unconnected {
    fn disconnect(self) -> Unconnected {
        eprintln!("Already disconnected from Walabot, ignoring");
        Unconnected
    }
}

impl Disconnect for Connected {}
