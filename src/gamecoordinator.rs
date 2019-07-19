use super::*;
use libc::c_void;

pub const STEAMGAMECOORDINATOR_INTERFACE_VERSION: &str = "SteamGameCoordinator001";

pub struct GameCoordinator<Manager> {
    pub(crate) gamcoord: *mut c_void,
    pub(crate) inner: Arc<Inner<Manager>>,
}

impl <Manager> GameCoordinator<Manager> {

    pub fn is_message_available(&self, msg_size: *mut u32) -> bool {
        unsafe {
            sys::SteamAPI_ISteamGameCoordinator_IsMessageAvailable(self.gamcoord, msg_size)
        }
    }

    pub fn retrieve_message(&self, pun_msg_type: *mut u32, pub_dest: *mut c_void, cub_dest: u32, pcub_msg_size: &mut u32) -> sys::EGCResults {
        unsafe {
            sys::SteamAPI_ISteamGameCoordinator_RetrieveMessage(
                self.gamcoord, 
                pun_msg_type,
                pub_dest,
                cub_dest, 
                pcub_msg_size)
        }
    }

    pub fn send_message(&self, un_msg_type: u32, pub_data: *const c_void, cub_data: u32) -> sys::EGCResults {
        unsafe {
            sys::SteamAPI_ISteamGameCoordinator_SendMessage(
                self.gamcoord, 
                un_msg_type, 
                pub_data, 
                cub_data)
        }
    }
}
