use git2::Repository;

/// Information about the current git state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitInfo {
    /// The commit hash (SHA-1) of the current HEAD
    pub commit_hash: String,
    /// The name of the current branch, or "detached HEAD" if not on a branch
    pub branch: String,
}

/// Retrieve git repository information (commit hash and branch)
///
/// Returns `Some(GitInfo)` if the repository is found and valid,
/// or `None` if there's any error (not a git repo, corrupted, etc.)
pub fn get_git_info() -> Option<GitInfo> {
    // Open the repository from the current directory
    let repo = Repository::discover(".").ok()?;

    // Get the HEAD reference
    let head = repo.head().ok()?;

    // Get the commit hash
    let commit = head.peel_to_commit().ok()?;
    let commit_hash = format!("{}", commit.id());

    // Determine branch name
    let branch = if head.is_branch() {
        head.shorthand().unwrap_or("unknown").to_string()
    } else {
        "detached HEAD".to_string()
    };

    Some(GitInfo {
        commit_hash,
        branch,
    })
}

/// Get a hash prefix for directory naming
///
/// Returns the first 8 characters of the commit hash if in a valid git repository
/// and on a branch. Returns "nogit" if:
/// - Not in a git repository
/// - In detached HEAD state
/// - Any error occurs
pub fn get_hash_prefix_for_directory() -> String {
    match get_git_info() {
        Some(git_info) => {
            // If detached HEAD, treat as nogit
            if git_info.branch == "detached HEAD" {
                "nogit".to_string()
            } else {
                // Get first 8 characters of the hash
                git_info.commit_hash.chars().take(8).collect()
            }
        }
        None => "nogit".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_git_repository_when_get_git_info_then_returns_some() {
        // This test assumes we're running in a git repository
        let result = get_git_info();

        // Should return Some since we're in a git repo
        assert!(
            result.is_some(),
            "Expected Some(GitInfo) when running in a git repository"
        );

        let git_info = result.unwrap();

        // Commit hash should be 40 hex characters (SHA-1)
        assert_eq!(
            git_info.commit_hash.len(),
            40,
            "Commit hash should be 40 characters"
        );
        assert!(
            git_info.commit_hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Commit hash should contain only hex digits"
        );

        // Branch should not be empty
        assert!(
            !git_info.branch.is_empty(),
            "Branch name should not be empty"
        );
    }

    #[test]
    fn given_git_info_when_created_then_fields_are_accessible() {
        let git_info = GitInfo {
            commit_hash: "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0".to_string(),
            branch: "main".to_string(),
        };

        assert_eq!(
            git_info.commit_hash,
            "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0"
        );
        assert_eq!(git_info.branch, "main");
    }

    #[test]
    fn given_git_info_when_cloned_then_values_match() {
        let original = GitInfo {
            commit_hash: "1234567890abcdef1234567890abcdef12345678".to_string(),
            branch: "feature/test".to_string(),
        };

        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(original.commit_hash, cloned.commit_hash);
        assert_eq!(original.branch, cloned.branch);
    }

    #[test]
    fn given_two_different_git_infos_when_compared_then_not_equal() {
        let info1 = GitInfo {
            commit_hash: "1234567890abcdef1234567890abcdef12345678".to_string(),
            branch: "main".to_string(),
        };

        let info2 = GitInfo {
            commit_hash: "abcdef1234567890abcdef1234567890abcdef12".to_string(),
            branch: "main".to_string(),
        };

        assert_ne!(info1, info2);
    }

    #[test]
    fn given_detached_head_when_formatted_then_shows_detached() {
        let git_info = GitInfo {
            commit_hash: "1234567890abcdef1234567890abcdef12345678".to_string(),
            branch: "detached HEAD".to_string(),
        };

        assert_eq!(git_info.branch, "detached HEAD");
    }

    #[test]
    fn given_git_repository_when_get_hash_prefix_then_returns_8_chars() {
        // This test assumes we're running in a git repository on a branch
        let prefix = get_hash_prefix_for_directory();

        // Should either be "nogit" or 8 hex characters
        if prefix == "nogit" {
            // If we're in detached HEAD or not in repo, this is fine
            assert_eq!(prefix, "nogit");
        } else {
            // Should be 8 hex characters
            assert_eq!(prefix.len(), 8);
            assert!(prefix.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn given_valid_git_info_when_on_branch_then_prefix_is_8_chars() {
        // We can't easily mock git2, but we can test the logic by checking
        // that if we're in a repo and get_git_info returns Some with a branch,
        // the prefix should be 8 characters
        if let Some(git_info) = get_git_info() {
            if git_info.branch != "detached HEAD" {
                let prefix = get_hash_prefix_for_directory();
                assert_eq!(prefix.len(), 8);
                assert!(prefix.chars().all(|c| c.is_ascii_hexdigit()));
                // Verify it matches the first 8 chars of the full hash
                let expected: String = git_info.commit_hash.chars().take(8).collect();
                assert_eq!(prefix, expected);
            }
        }
    }
}
