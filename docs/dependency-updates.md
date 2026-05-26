# Automated Dependency Update Workflow

## Overview

This project uses Dependabot to automatically detect and propose updates for Rust dependencies. The workflow is configured to:

- Detect outdated dependencies weekly
- Group updates by type (patch, minor, major)
- Auto-merge patch updates that pass CI
- Require manual review for major/minor updates

## Configuration

The Dependabot configuration is defined in `.github/dependabot.yml`.

### Update Schedule

- **Frequency**: Weekly
- **Day**: Monday
- **Time**: 03:00 UTC
- **Max Open PRs**: 10

### Dependency Groups

Updates are grouped into three categories:

#### 1. Patch Updates (Auto-merge)

- **Type**: Patch version bumps (e.g., 1.2.3 → 1.2.4)
- **Auto-merge**: Yes (if CI passes)
- **Merge method**: Squash
- **Review**: Automatic

Patch updates are automatically merged when:
- All CI checks pass
- No conflicts exist
- Direct dependencies only

#### 2. Minor Updates (Manual Review)

- **Type**: Minor version bumps (e.g., 1.2.0 → 1.3.0)
- **Auto-merge**: No
- **Review**: Required
- **Action**: Create PR for manual review

Minor updates may introduce new features but maintain backward compatibility.

#### 3. Major Updates (Manual Review)

- **Type**: Major version bumps (e.g., 1.0.0 → 2.0.0)
- **Auto-merge**: No
- **Review**: Required
- **Action**: Create individual PR for each major update

Major updates may introduce breaking changes and require careful review.

## Pinned Dependencies

Some dependencies are intentionally pinned to specific versions:

| Dependency | Version | Reason |
|-----------|---------|--------|
| `home` | `0.5.11` | Stability and compatibility |

To add a pinned dependency:

1. Update `.github/dependabot.yml` in the `ignore` section
2. Document the reason in this file
3. Create an issue to track when the pin can be removed

## Workflow

### Automatic Patch Updates

1. Dependabot detects a patch update
2. Creates a PR with the update
3. CI runs automatically
4. If CI passes, PR is auto-merged
5. Commit is added to develop branch

### Manual Review Updates

1. Dependabot detects a minor/major update
2. Creates a PR with the update
3. PR is assigned to maintainers
4. Maintainers review:
   - Changelog for breaking changes
   - Test results
   - Compatibility with codebase
5. Maintainers approve and merge manually

## PR Labels and Metadata

All Dependabot PRs include:

- **Labels**: `dependencies`, `rust`
- **Assignees**: `synapse-bridgez/maintainers`
- **Reviewers**: `synapse-bridgez/maintainers`
- **Commit prefix**: `chore(deps)` or `chore(deps-dev)`

## Handling Conflicts

If a Dependabot PR has conflicts:

1. Dependabot automatically rebases the PR
2. If rebase fails, manual intervention is needed
3. Maintainers can:
   - Manually resolve conflicts
   - Close and recreate the PR
   - Update the dependency manually

## Disabling Updates

To temporarily disable Dependabot:

1. Go to repository Settings
2. Navigate to Code security and analysis
3. Disable Dependabot

To disable for specific dependencies:

1. Update `.github/dependabot.yml`
2. Add to `ignore` section:
   ```yaml
   ignore:
     - dependency-name: "package-name"
   ```

## Monitoring

### Metrics

Track the following metrics:

- Number of PRs created per week
- Auto-merge success rate
- Time to merge manual review PRs
- Number of failed CI checks

### Alerts

Set up alerts for:

- High number of outdated dependencies
- Repeated CI failures
- Stale PRs (not merged within 7 days)

## Best Practices

1. **Review regularly**: Check Dependabot PRs at least weekly
2. **Merge patch updates promptly**: Keep patch updates current
3. **Test major updates thoroughly**: Don't rush major version bumps
4. **Document breaking changes**: Update CHANGELOG when merging major updates
5. **Keep CI green**: Ensure all tests pass before merging

## Troubleshooting

### Dependabot Not Creating PRs

**Symptoms**: No PRs created for outdated dependencies

**Causes**:
- Dependabot is disabled
- No outdated dependencies detected
- Configuration error

**Solution**:
1. Check repository settings
2. Verify `.github/dependabot.yml` syntax
3. Check Dependabot logs in Security tab

### Auto-merge Not Working

**Symptoms**: Patch updates not auto-merging despite passing CI

**Causes**:
- Branch protection rules blocking merge
- Required status checks not configured
- Auto-merge disabled in settings

**Solution**:
1. Check branch protection rules
2. Verify required status checks
3. Enable auto-merge in repository settings

### Frequent Conflicts

**Symptoms**: Many Dependabot PRs have merge conflicts

**Causes**:
- Frequent manual dependency updates
- Conflicting changes in Cargo.toml
- Multiple Dependabot PRs for same dependency

**Solution**:
1. Coordinate manual updates with Dependabot
2. Merge Dependabot PRs more frequently
3. Reduce update frequency if necessary

## References

- [Dependabot Documentation](https://docs.github.com/en/code-security/dependabot)
- [Dependabot Configuration Options](https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuration-options-for-dependency-updates)
- [Cargo.toml Format](https://doc.rust-lang.org/cargo/reference/manifest.html)
- [Semantic Versioning](https://semver.org/)
