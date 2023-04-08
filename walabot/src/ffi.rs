use uom::si::f64::Power;
use std::ffi::{c_char, c_uchar, c_int, c_double};

use super::*;

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct AntennaID(c_int);

impl AntennaID {
    pub fn new(id: i32) -> Self {
        AntennaID(id)
    }
}

/// Bridge to the Walabot shared lib
#[derive(Deref, Debug)]
pub struct WalabotAPI(Library);

impl From<Library> for WalabotAPI {
    fn from(lib: Library) -> Self {
        WalabotAPI(lib)
    }
}


#[derive(Debug, Copy, Clone, Display)]
#[repr(C)]
pub enum Status {
    Clean,
    Initialized,
    Connected,
    Configured,
    Scanning,
    /// Calibrating, param is percent complete
    Calibrating(f64),
    /// Mystery state
    CalibratingNoMovement(f64),
}

impl From<c_int> for Status {
    fn from(rawstatus: c_int) -> Self {
        match rawstatus {
            0 => Status::Clean,
            1 => Status::Initialized,
            2 => Status::Connected,
            3 => Status::Configured,
            4 => Status::Scanning,
            5 => Status::Calibrating(0.0),
            6 => Status::CalibratingNoMovement(0.0),
            _ => panic!("Unknown status: {}", rawstatus),
        }
    }
}

impl Status {
    pub fn and(self, param: f64) -> Result<Self> {
        match self {
            Status::Calibrating(_) => Ok(Status::Calibrating(param)),
            Status::CalibratingNoMovement(_) => Ok(Status::CalibratingNoMovement(param)),
            _ => Ok(self),
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub enum TargetKind {
    Unknown,
    Pipe,
    Stud,
    Stud90,
    MetalStud,
    Other
}


#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Target {
    x: Length,
    y: Length,
    z: Length,
    amplitude: Power,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ImagingTarget {
    kind: TargetKind,
    angle: Angle,
    x: Length,
    y: Length,
    z: Length,
    width: Length,
    amplitude: Power,
}


#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct AntennaPair {
    pub tx_antenna: AntennaID,
    pub rx_antenna: AntennaID,
}

/// Scanning profiles (sensor configuration)
#[derive(Debug, Default, Copy, Clone)]
#[repr(C)]
pub enum ScanProfile {
    #[default]
    ShortRange = 0x0001_0000,
    LongRange = 0x0002_0000,
    Tracker = 0x0003_0000,
    WideBand = 0x0004_0000,
}

#[derive(Debug, Default, Copy, Clone)]
#[repr(C)]
pub enum FilterKind {
    #[default]
    None,
    /// Dyanamic-imaging filter for breathing frequencies
    Derivative,
    /// Moving Target Identification
    MTI,
}

pub type ResultType = c_int;

#[link(name = "WalabotAPI")]
extern "C" {
    #![allow(non_snake_case)]
    pub fn Walabot_Initialize(configFilename: *const c_char) -> ResultType;
    pub fn Walabot_Clean() -> ResultType;
    pub fn Walabot_GetExtendedError() -> ResultType;
    pub fn Walabot_GetErrorString() -> *const c_char;
    pub fn Walabot_SetSettingsFolder(path: *const c_uchar) -> ResultType;
    pub fn Walabot_GetInstrumentsList(buf_size: c_int, buf: *mut c_char) -> ResultType;
    pub fn Walabot_Connect(uid: *const c_char) -> ResultType;
    pub fn Walabot_ConnectAny() -> ResultType;
    pub fn Walabot_Disconnect() -> ResultType;
    pub fn Walabot_Start() -> ResultType;
    pub fn Walabot_Stop() -> ResultType;
    pub fn Walabot_Trigger() -> ResultType;
    pub fn Walabot_GetAntennaPairs(antenna_pairs: *const *const AntennaPair, num_pairs: *const c_int) -> ResultType;
    pub fn Walabot_GetSignal(tx_antenna: c_int, rx_antenna: c_int, signal: *const *const c_double, timeAxis: *const *const c_double, numSamples: *const c_int) -> ResultType;
    pub fn Walabot_SetProfile(profile: ScanProfile) -> ResultType;
    pub fn Walabot_GetStatus(status: *mut Status, param: *const c_double) -> ResultType;
    pub fn Walabot_StartCalibration() -> ResultType;
    pub fn Walabot_CancelCalibration() -> ResultType;
    pub fn Walabot_SetThreshold(value: c_double) -> ResultType;
    pub fn Walabot_SetTrackerAquisitionThreshold( value: c_double) -> ResultType;
    pub fn Walabot_GetThreshold(threshold: *mut c_double) -> ResultType;
    pub fn Walabot_SetArenaX(min: c_double, max: c_double, res: c_double) -> ResultType;
    pub fn Walabot_SetArenaY(min: c_double, max: c_double, res: c_double) -> ResultType;
    pub fn Walabot_SetArenaZ(start: c_double, end: c_double, res: c_double) -> ResultType;
    pub fn Walabot_SetArenaR(start: c_double, end: c_double, res: c_double) -> ResultType;
    pub fn Walabot_SetArenaTheta(min: c_double, max: c_double, res: c_double) -> ResultType;
    pub fn Walabot_SetArenaPhi(min: c_double, max: c_double, res: c_double) -> ResultType;
    pub fn Walabot_GetArenaX(min: *mut c_double, max: *mut c_double, res: *mut c_double) -> ResultType;
    pub fn Walabot_GetArenaY(min: *mut c_double, max: *mut c_double, res: *mut c_double) -> ResultType;
    pub fn Walabot_GetArenaZ(start: *mut c_double, end: *mut c_double, res: *mut c_double) -> ResultType;
    pub fn Walabot_GetArenaR(start: *mut c_double, end: *mut c_double, res: *mut c_double) -> ResultType;
    pub fn Walabot_GetArenaTheta(min: *mut c_double, max: *mut c_double, res: *mut c_double) -> ResultType;
    pub fn Walabot_GetArenaPhi(min: *mut c_double, max: *mut c_double, res: *mut c_double) -> ResultType;
    pub fn Walabot_GetRawImageSlice(rasterImage: *mut *const c_int, sizeX: *mut c_int, sizeY: *mut c_int, sizeZ: *mut c_int, sliceDepth: *mut c_double, power: *mut c_double) -> ResultType;
    pub fn Walabot_GetRawImage(rasterImage: *mut *const c_int, sizeX: *mut c_int, sizeY: *mut c_int, sizeZ: *mut c_int, power: *mut c_double) -> ResultType;
    pub fn Walabot_GetImageEnergy(energy: *mut c_double) -> ResultType;
    pub fn Walabot_GetImagingTargets(targets: *mut *const TargetKind, numTargets: *mut c_int) -> ResultType;
    pub fn Walabot_GetSensorTargets(targets: *mut *const Target, numTargets: *mut c_int) -> ResultType;
    pub fn Walabot_GetTrackerTargets(targets: *mut *const Target, numTargets: *mut c_int) -> ResultType;
    pub fn Walabot_SetDynamicImageFilter(kind: *const FilterKind) -> ResultType;
    pub fn Walabot_GetDynamicImageFilter(kind: *mut FilterKind) -> ResultType;
    pub fn Walabot_GetVersion(version: *const *mut c_char) -> ResultType;
    pub fn Walabot_SetAdvancedParameter(paramName: *const c_char, value: c_double) -> ResultType;
    pub fn Walabot_GetAdvancedParameter(paramName: *const *mut c_char, value: *mut c_double) -> ResultType;
    pub fn Walabot_GetAntennaLocation(antennaNum: AntennaID, X: *mut c_double, Y: *mut c_double, Z: *mut c_double) -> ResultType;
}