use std::{cmp::Ordering, convert::TryFrom};
use getset::Getters;

use crate::structures::errors::{ParseError, PermissionParseError};

pub trait Permissible<T> {
    fn matches(&self, perm_node: T) -> bool;
}

#[derive(Debug, PartialEq, Default, Clone, new, Getters)]
#[getset(get = "pub")]
pub struct Permission {
    node: String,
    wildcard: bool,
}

impl Permissible<&str> for Permission {
    fn matches(&self, perm_node: &str) -> bool {
        if !self.wildcard {
            return self.node.chars().rev().cmp(perm_node.chars().rev()) == Ordering::Equal;
        }
        perm_node
            .chars()
            .nth(self.node.len())
            .filter(|c| c == &'.')
            .is_some()
            && perm_node.starts_with(&*self.node)
    }
}

impl Permissible<&String> for Permission {
    fn matches(&self, perm_node: &String) -> bool {
        self.matches(perm_node.as_str())
    }
}

impl TryFrom<&String> for Permission {
    type Error = PermissionParseError;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let len = value.len();
        if len == 0 { return Err(PermissionParseError::InvalidPermissionString(value.clone())) }
        let last_char = value.chars().last();
        if last_char == Some('.') { return Err(PermissionParseError::InvalidPermissionString(value.clone())) }
        let wildcard = last_char == Some('*');
        let node = if wildcard {
            if len >= 2 && value.chars().nth(len - 2) == Some('.') {
                value[..len-2].to_string()
            } else {
                return Err(PermissionParseError::InvalidPermissionString(value.clone()))
            }
        } else {
            value.clone()
        };
        return Ok(Permission { node, wildcard })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use pretty_assertions::assert_eq;

    #[rstest]
    #[case("rootnode")]
    #[case("my.perm")]
    #[case("my.long.perm.node")]
    fn exact_wildcardless_matching(#[case] perm_node: &str) {
        let perm = Permission {
            node: perm_node.to_string(),
            wildcard: false,
        };
        assert!(perm.matches(perm_node), "Didn't match perm for {:?}", perm_node);
    }

    #[rstest]
    #[case("root", "rootnode")]
    #[case("my", "my.perm")]
    #[case("my.long", "my.long.perm.node")]
    fn prefix_wildcardless_no_matching(#[case] node: String, #[case] perm_node: &str) {
        let perm = Permission {
            node,
            wildcard: false,
        };
        assert!(
            !perm.matches(perm_node),
            "Did match {:?} perm for {:?}",
            perm,
            perm_node
        );
    }

    #[rstest]
    #[case("root", "rootnode")]
    #[case("my.p", "my.perm")]
    #[case("my.lo", "my.long.perm.node")]
    fn prefix_wildcard_no_matching(#[case] node: String, #[case] perm_node: &str) {
        let perm = Permission { node, wildcard: true };
        assert!(
            !perm.matches(perm_node),
            "Did match {:?} perm for {:?}",
            perm,
            perm_node
        );
    }

    #[rstest]
    #[case("my", "my.perm")]
    #[case("my.long", "my.long.perm.node")]
    fn prefix_wildcard_matching(#[case] node: String, #[case] perm_node: &str) {
        let perm = Permission { node, wildcard: true };
        assert!(
            perm.matches(perm_node),
            "Didn't match {:?} perm for {:?}",
            perm,
            perm_node
        );
    }

    #[rstest]
    #[case::simple_dotless("root", "root", false)]
    #[case::simple("my.perm", "my.perm", false)]
    #[case::simple_long("my.long.perm.node", "my.long.perm.node", false)]
    #[case::wildcard("my.*", "my", true)]
    #[case::wildcard_long("my.long.perm.*", "my.long.perm", true)]
    fn check_successful_permission_parsing(#[case] serialized: String, #[case] node: String, #[case] wildcard: bool) {
        let result = Permission::try_from(&serialized);
        let expected = Permission { node, wildcard };
        match result {
            Ok(deserialized) => assert_eq!(deserialized, expected),
            Err(e) => panic!("Expected {} to deserialize succesfully to {:?}, got error instead: {:?}", serialized, expected, e),
        }
    }

    #[rstest]
    #[case::empty("")]
    #[case::dot(".")]
    #[case::wildcard("*")]
    #[case::wildcard_without_dot("myperm*")]
    #[case::wildcard_before_dot("myperm*.")]
    fn check_invalid_permission_parsing(#[case] serialized: String) {
        let result = Permission::try_from(&serialized);
        match result {
            Ok(deserialized) => panic!("Expected {} to fail to deserialize, got instead: {:?}", serialized, deserialized),
            Err(_e) => {},
        }
    }
}
