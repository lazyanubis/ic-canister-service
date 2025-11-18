use candid::CandidType;
use ic_canister_kit::types::Permission;
use serde::{Deserialize, Serialize};

mod common;
pub use common::*;

mod business;
pub use business::*;

// ==================== 更新版本需要修改下面代码 ====================

mod v000;
mod v001;

// ! 此处应该是最新的版本
// !     👇👇 UPGRADE WARNING: 必须是当前代码的版本
pub use v001::types::*;

pub enum State {
    V0(Box<v000::types::InnerState>),
    V1(Box<v001::types::InnerState>),
    // *    👆👆 UPGRADE WARNING: 引入新版本
}
use State::*;

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum InitArgs {
    V0(Box<v000::types::InitArg>),
    V1(Box<v001::types::InitArg>),
    // *    👆👆 UPGRADE WARNING: 引入新版本
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum UpgradeArgs {
    V0(Box<v000::types::UpgradeArg>),
    V1(Box<v001::types::UpgradeArg>),
    // *    👆👆 UPGRADE WARNING: 引入新版本
}

// 初始化
impl Initial<Option<InitArgs>> for State {
    fn init(&mut self, args: Option<InitArgs>) {
        match args {
            Some(args) => match (self, args) {
                (V0(s), InitArgs::V0(arg)) => s.init(Some(*arg)),
                (V1(s), InitArgs::V1(arg)) => s.init(Some(*arg)),
                // ! 👆👆 新增版本需要添加默认的数据
                _ => ic_cdk::trap("version mismatched"),
            },
            None => match self {
                V0(s) => s.init(None),
                V1(s) => s.init(None),
            },
        }
    }
}

// 升级版本
impl Upgrade<Option<UpgradeArgs>> for State {
    fn upgrade(&mut self, args: Option<UpgradeArgs>) {
        'outer: loop {
            // 进行升级操作, 不断地升到下一版本
            match self {
                V0(s) => *self = V1(std::mem::take(&mut *s).into()), // -> V1
                V1(_) => break 'outer,                               // same version do nothing
            }
        }

        // handle args
        match args {
            Some(args) => {
                match (self, args) {
                    (V0(s), UpgradeArgs::V0(arg)) => s.upgrade(Some(*arg)),
                    (V1(s), UpgradeArgs::V1(arg)) => s.upgrade(Some(*arg)),
                    // ! 👆👆 新增版本需要添加默认的数据
                    _ => ic_cdk::trap("version mismatched"),
                }
            }
            None => match self {
                V0(s) => s.upgrade(None),
                V1(s) => s.upgrade(None),
            },
        }
    }
}

impl StateUpgrade<Option<UpgradeArgs>> for State {
    fn version(&self) -> u32 {
        // 每个版本的版本号
        match self {
            V0(_) => 0,
            V1(_) => 1,
            // *   👆👆! 升级需要在此添加版本号
        }
    }

    fn from_version(version: u32) -> Self {
        match version {
            0 => V0(Box::default()), // * 初始化
            1 => V1(Box::default()), // * 初始化
            // ! 👆👆 新增版本需要添加默认的数据
            _ => ic_cdk::trap("unsupported version"),
        }
    }
}

// ================== get ==================

impl business::immutable::GetImmutable for State {
    fn get(&self) -> &dyn Business {
        match self {
            V0(s) => s.as_ref(), // * 获取不可变对象
            V1(s) => s.as_ref(), // * 获取不可变对象
        }
    }
}

impl business::mutable::GetMutable for State {
    fn get_mut(&mut self) -> &mut dyn MutableBusiness {
        match self {
            V0(s) => s.as_mut(), // * 获取可变对象
            V1(s) => s.as_mut(), // * 获取可变对象
        }
    }
}

// ================== 权限 ==================
// 本罐子需要的权限转换
pub trait ParsePermission {
    fn parse_permission<'a>(&self, name: &'a str) -> Result<Permission, ParsePermissionError<'a>>;
}
#[derive(Debug, Clone, Serialize, CandidType)]
pub struct ParsePermissionError<'a>(&'a str);
impl Display for ParsePermissionError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParsePermissionError: {}", self.0)
    }
}
impl std::error::Error for ParsePermissionError<'_> {}
