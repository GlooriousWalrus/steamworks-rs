
use super::*;
use std::net::Ipv4Addr;

const CALLBACK_BASE_ID: i32 = 300;

bitflags! {
    #[repr(C)]
    pub struct FriendFlags: u16 {
        const NONE                  = 0x0000;
        const BLOCKED               = 0x0001;
        const FRIENDSHIP_REQUESTED  = 0x0002;
        const IMMEDIATE             = 0x0004;
        const CLAN_MEMBER           = 0x0008;
        const ON_GAME_SERVER        = 0x0010;
        // Unused
        // Unused
        const REQUESTING_FRIENDSHIP = 0x0080;
        const REQUESTING_INFO       = 0x0100;
        const IGNORED               = 0x0200;
        const IGNORED_FRIEND        = 0x0400;
        // Unused
        const CHAT_MEMBER           = 0x1000;
        const ALL                   = 0xFFFF;
    }
}

bitflags! {
    #[repr(C)]
    pub struct PersonaChange: i32 {
        const NAME                = 0x0001;
        const STATUS              = 0x0002;
        const COME_ONLINE         = 0x0004;
        const GONE_OFFLINE        = 0x0008;
        const GAME_PLAYED         = 0x0010;
        const GAME_SERVER         = 0x0020;
        const AVATAR              = 0x0040;
        const JOINED_SOURCE       = 0x0080;
        const LEFT_SOURCE         = 0x0100;
        const RELATIONSHIP_CHANGE = 0x0200;
        const NAME_FIRST_SET      = 0x0400;
        const FACEBOOK_INFO       = 0x0800;
        const NICKNAME            = 0x1000;
        const STEAM_LEVEL         = 0x2000;
    }
}

/// Access to the steam friends interface
pub struct Friends<Manager> {
    pub(crate) friends: *mut sys::ISteamFriends,
    pub(crate) inner: Arc<Inner<Manager>>,
}

impl <Manager> Friends<Manager> {

    /// Returns the (display) name of the current user
    pub fn name(&self) -> String {
        unsafe {
            let name = sys::SteamAPI_ISteamFriends_GetPersonaName(self.friends);
            let name = CStr::from_ptr(name);
            name.to_string_lossy().into_owned()
        }
    }

    pub fn get_friends(&self, flags: FriendFlags) -> Vec<Friend<Manager>> {
        unsafe {
            let count = sys::SteamAPI_ISteamFriends_GetFriendCount(self.friends, flags.bits() as _);
            if count == -1 {
                return Vec::new();
            }
            let mut friends = Vec::with_capacity(count as usize);
            for idx in 0 .. count {
                let friend = SteamId(sys::SteamAPI_ISteamFriends_GetFriendByIndex(self.friends, idx, flags.bits() as _).0);
                friends.push(Friend {
                    id: friend,
                    friends: self.friends,
                    _inner: self.inner.clone(),
                });
            }

            friends
        }
    }

    pub fn get_friend(&self, friend: SteamId) -> Friend<Manager> {
        Friend {
            id: friend,
            friends: self.friends,
            _inner: self.inner.clone(),
        }
    }

    pub fn request_user_information(&self, user: SteamId, name_only: bool) {
        unsafe {
            sys::SteamAPI_ISteamFriends_RequestUserInformation(self.friends, sys::CSteamID(user.0), name_only as u8);
        }
    }

    // I don't know why this is part of friends either
    pub fn activate_game_overlay_to_web_page(&self, url: &str) {
        unsafe {
            let url = CString::new(url).unwrap();
            sys::SteamAPI_ISteamFriends_ActivateGameOverlayToWebPage(self.friends, url.as_ptr() as *const _);
        }
    }

    /// Opens up an invite dialog for the given lobby
    pub fn activate_invite_dialog(&self, lobby: LobbyId) {
        unsafe {
            sys::SteamAPI_ISteamFriends_ActivateGameOverlayInviteDialog(self.friends, sys::CSteamID(lobby.0));
        }
    }
}

/// Information about a friend's current state in a game
#[derive(Debug)]
pub struct FriendGame {
    /// The id of the game that the friend is
    /// playing
    pub game: GameId,
    /// The address of the server the player is in
    pub game_address: Ipv4Addr,
    /// The game port of the server the player is in
    pub game_port: u16,
    /// The query port of the server the player is in
    pub query_port: u16,
    /// Optional id of the lobby the player is in
    pub lobby: LobbyId,
}

#[derive(Debug)]
pub struct PersonaStateChange {
    pub steam_id: SteamId,
    pub flags: PersonaChange,
}

unsafe impl Callback for PersonaStateChange {
    const ID: i32 = CALLBACK_BASE_ID + 4;
    const SIZE: i32 = ::std::mem::size_of::<sys::PersonaStateChange_t>() as i32;

    unsafe fn from_raw(raw: *mut libc::c_void) -> Self {
        let val = &mut *(raw as *mut sys::PersonaStateChange_t);
        PersonaStateChange {
            steam_id: SteamId(val.m_ulSteamID),
            flags: PersonaChange::from_bits_truncate(val.m_nChangeFlags as i32),
        }
    }
}

#[derive(Debug)]
pub struct GameLobbyJoinRequested {
    pub lobby_steam_id: LobbyId,
    pub friend_steam_id: SteamId,
}

unsafe impl Callback for GameLobbyJoinRequested {
    const ID: i32 = CALLBACK_BASE_ID + 33;
    const SIZE: i32 = ::std::mem::size_of::<sys::GameLobbyJoinRequested_t>() as i32;

    unsafe fn from_raw(raw: *mut libc::c_void) -> Self {
        let val = &mut *(raw as *mut sys::GameLobbyJoinRequested_t);
        GameLobbyJoinRequested {
            lobby_steam_id: LobbyId(val.m_steamIDLobby.0),
            friend_steam_id: SteamId(val.m_steamIDFriend.0),
        }
    }
}

pub struct Friend<Manager> {
    id: SteamId,
    friends: *mut sys::ISteamFriends,
    _inner: Arc<Inner<Manager>>,
}

impl <Manager> Debug for Friend<Manager> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Friend({:?})", self.id)
    }
}

impl <Manager> Friend<Manager> {
    pub fn id(&self) -> SteamId {
        self.id
    }

    pub fn name(&self) -> String {
        unsafe {
            let name = sys::SteamAPI_ISteamFriends_GetFriendPersonaName(self.friends, sys::CSteamID(self.id.0));
            let name = CStr::from_ptr(name);
            name.to_string_lossy().into_owned()
        }
    }

    pub fn state(&self) -> FriendState {
        unsafe {
            let state = sys::SteamAPI_ISteamFriends_GetFriendPersonaState(self.friends, sys::CSteamID(self.id.0));
            match state {
                sys::EPersonaState::EPersonaStateOffline => FriendState::Offline,
                sys::EPersonaState::EPersonaStateOnline => FriendState::Online,
                sys::EPersonaState::EPersonaStateBusy => FriendState::Busy,
                sys::EPersonaState::EPersonaStateAway => FriendState::Away,
                sys::EPersonaState::EPersonaStateSnooze => FriendState::Snooze,
                sys::EPersonaState::EPersonaStateLookingToPlay => FriendState::LookingToPlay,
                sys::EPersonaState::EPersonaStateLookingToTrade => FriendState::LookingToTrade,
                _ => unreachable!(),
            }
        }
    }

    /// Returns information about the game the player is current playing if any
    pub fn game_played(&self) -> Option<FriendGame> {
        unsafe {
            let mut info: sys::FriendGameInfo_t = std::mem::uninitialized();
            if sys::SteamAPI_ISteamFriends_GetFriendGamePlayed(self.friends, sys::CSteamID(self.id.0), &mut info) != 0 {
                Some(FriendGame {
                    game: GameId(info.m_gameID.0),
                    game_address: info.m_unGameIP.into(),
                    game_port: info.m_usGamePort,
                    query_port: info.m_usQueryPort,
                    lobby: LobbyId(info.m_steamIDLobby.0),
                })
            } else {
                None
            }
        }
    }

    /// Returns a small (32x32) avatar for the user
    pub fn small_avatar(&self) -> Option<Vec<u8>> {
        unsafe {
            let utils = sys::steam_rust_get_utils();
            let img = sys::SteamAPI_ISteamFriends_GetSmallFriendAvatar(self.friends, sys::CSteamID(self.id.0));
            if img == 0 {
                return None;
            }
            let mut width = 0;
            let mut height = 0;
            if sys::SteamAPI_ISteamUtils_GetImageSize(utils, img, &mut width, &mut height) == 0 {
                return None;
            }
            assert_eq!(width, 32);
            assert_eq!(height, 32);
            let mut dest = vec![0; 32 * 32 * 4];
            if sys::SteamAPI_ISteamUtils_GetImageRGBA(utils, img, dest.as_mut_ptr(), 32 * 32 * 4) == 0 {
                return None;
            }
            Some(dest)
        }
    }

    /// Returns a small (64x64) avatar for the user
    pub fn medium_avatar(&self) -> Option<Vec<u8>> {
        unsafe {
            let utils = sys::steam_rust_get_utils();
            let img = sys::SteamAPI_ISteamFriends_GetMediumFriendAvatar(self.friends, sys::CSteamID(self.id.0));
            if img == 0 {
                return None;
            }
            let mut width = 0;
            let mut height = 0;
            if sys::SteamAPI_ISteamUtils_GetImageSize(utils, img, &mut width, &mut height) == 0 {
                return None;
            }
            assert_eq!(width, 64);
            assert_eq!(height, 64);
            let mut dest = vec![0; 64 * 64 * 4];
            if sys::SteamAPI_ISteamUtils_GetImageRGBA(utils, img, dest.as_mut_ptr(), 64 * 64 * 4) == 0 {
                return None;
            }
            Some(dest)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum FriendState {
    Offline,
    Online,
    Busy,
    Away,
    Snooze,
    LookingToTrade,
    LookingToPlay,
}