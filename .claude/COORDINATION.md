# Wave Coordination Guide

## Overview

Each wave runs in its own Claude Code session. Sessions coordinate via `WAVES.org` and update their prompt files with discoveries.

## Starting a New Wave Session

### 1. Check Dependencies

Open `WAVES.org` and verify:

- Dependencies are marked COMPLETE
- Status shows READY or NOT STARTED (not BLOCKED)

### 2. Create Session

Start a new Claude Code session with this initial message:

```
I'm working on Wave [N] - [Wave Name] for the Bits Wizard of Oz launch.

Please:
1. Read WAVES.org and find my wave section
2. Read my prompt file (linked in WAVES.org)
3. Update WAVES.org to mark my wave as IN PROGRESS
4. Reflect on the prompt - research, validate approach, propose improvements
5. Update the prompt file with your improvements/discoveries
6. Ask me if I'm ready to proceed with implementation

My wave is: [copy the wave section from WAVES.org]
```

### 3. Session Workflow

The session will:

- ✅ Read `WAVES.org` and the prompt file(s)
- ✅ Reflect and research the approach
- ✅ Propose improvements/validate assumptions
- ✅ Ask for approval to proceed
- ✅ Implement the work
- ✅ Update prompt file with discoveries
- ✅ Update `WAVES.org` with progress
- ✅ Mark complete when done

## Example: Starting Wave 1, Prompt 1

**Your message to new session:**

```
I'm working on Wave 1, Prompt 1 - Terraform modules for Neon and Hetzner.

Please:
1. Read WAVES.org and find the "Prompt 1: Terraform modules" section
2. Read the prompt file: .claude/prompts/20251204134034-terraform-modules-for-neon-and-hetzner.org
3. Update WAVES.org to mark my wave as IN PROGRESS and add this session info
4. Reflect on the Terraform approach - research providers, validate architecture
5. Update the prompt file with any improvements you discover
6. Ask me if I'm ready to proceed with implementation

This wave has no dependencies and can start immediately.
```

## Running Parallel Waves

Wave 1 and Wave 3 have multiple prompts that can run in parallel. You can:

**Option A: Multiple Terminal Windows**

- Open 2-3 Claude Code sessions simultaneously
- Start each with its specific prompt
- They coordinate via `WAVES.org` (sessions can read/write)

**Option B: Sequential (simpler)**

- Complete one prompt at a time
- Less coordination overhead
- Takes longer but easier to track

**Recommendation:** Start with Option B (sequential) unless you're confident with parallel coordination.

## When a Wave Completes

**Session should:**

1. Update its section in `WAVES.org`:
   - Status: COMPLETE
   - Timestamp: completion time
   - Final discoveries summary
2. Update dependent waves:
   - Change BLOCKED → READY
   - Add note: "✅ Ready - Wave N complete"
3. Final prompt update with all learnings

**You should:**

1. Review the completed work
2. Check `WAVES.org` to see what's now READY
3. Start next wave session(s)

## Handling Blockers

If a session discovers it's blocked:

**Session should:**

1. Update `WAVES.org`:
   - Status: BLOCKED
   - Add reason: "Blocked: Missing X from Wave Y"
2. Update prompt with what's needed
3. Pause work gracefully

**You should:**

1. Address the blocker
2. Update `WAVES.org` when resolved
3. Resume or restart the session

## Session Instructions Template

Copy this for each wave session:

### Wave 1 Sessions

**Prompt 1 (Terraform):**

```
Working on: Wave 1, Prompt 1 - Terraform modules

Steps:
1. Read WAVES.org section "Prompt 1: Terraform modules"
2. Read prompt: .claude/prompts/20251204134034-terraform-modules-for-neon-and-hetzner.org
3. Update WAVES.org: status=IN PROGRESS, session=[this session]
4. Reflect and research: Hetzner/Neon/Cloudflare providers, NixOS config
5. Propose improvements to prompt
6. Get approval, then implement
7. Update prompt with discoveries
8. Mark COMPLETE in WAVES.org

Can run in PARALLEL with Prompt 2 (Container build).
```

**Prompt 2 (Container):**

```
Working on: Wave 1, Prompt 2 - Container build

Steps:
1. Read WAVES.org section "Prompt 2: Container build"
2. Read prompt: .claude/prompts/20251204134036-container-build-with-nix-and-docker.org
3. Update WAVES.org: status=IN PROGRESS, session=[this session]
4. Reflect and research: Nix Docker builds, Dioxus bundling
5. Propose improvements to prompt
6. Get approval, then implement
7. Update prompt with discoveries
8. Mark COMPLETE in WAVES.org

Can run in PARALLEL with Prompt 1 (Terraform).
```

### Wave 2 Session

```
Working on: Wave 2, Prompt 3 - Handle and Realm updates

Steps:
1. Read WAVES.org section "Prompt 3: Handle and Realm updates"
2. Read prompt: .claude/prompts/20251204134038-handle-type-and-realm-enum-updates.org
3. Verify dependencies: None (but BLOCKS Wave 3)
4. Update WAVES.org: status=IN PROGRESS
5. Reflect and research: Rust newtype pattern, Dioxus server functions
6. Propose improvements
7. Get approval, then implement
8. Verify: cargo check passes, all tests pass
9. Update prompt with discoveries
10. Mark COMPLETE in WAVES.org
11. Unblock Wave 3: Mark all Wave 3 prompts as READY

IMPORTANT: This blocks Wave 3 - complete before starting Wave 3!
```

### Wave 3 Sessions

**Prompt 4 (Demos):**

```
Working on: Wave 3, Prompt 4 - Demo profile system

Steps:
1. Check WAVES.org: Verify Wave 2 is COMPLETE
2. Read prompt: .claude/prompts/20251204134040-demo-profile-system.org
3. Update WAVES.org: status=IN PROGRESS
4. Reflect: Demo design, Tailwind styling, image URLs
5. Propose improvements
6. Implement demos/ directory
7. Test all demos render
8. Update prompt with discoveries
9. Mark COMPLETE in WAVES.org

Can run in PARALLEL with Prompts 5 (Subdomain) and 6 (Pages).
```

**Prompt 5 (Subdomain):**

```
Working on: Wave 3, Prompt 5 - Subdomain availability checker

Steps:
1. Check WAVES.org: Verify Wave 2 is COMPLETE
2. Read prompt: .claude/prompts/20251204134042-subdomain-availability-checker.org
3. Update WAVES.org: status=IN PROGRESS
4. Reflect: Handle validation, easter eggs, database queries
5. Propose improvements (add more easter eggs!)
6. Implement subdomain.rs
7. Test validation + easter eggs
8. Update prompt with discoveries
9. Mark COMPLETE in WAVES.org

Can run in PARALLEL with Prompts 4 (Demos) and 6 (Pages).
```

**Prompt 6 (Pages):**

```
Working on: Wave 3, Prompt 6 - Page component organization

Steps:
1. Check WAVES.org: Verify Wave 2 is COMPLETE
2. Read prompt: .claude/prompts/20251204134044-page-component-organization.org
3. Update WAVES.org: status=IN PROGRESS
4. Reflect: Module structure, re-exports, CLAUDE.md guidelines
5. Propose improvements
6. Create pages/ directory and refactor
7. Verify: cargo check passes, dev server works
8. Update prompt with discoveries
9. Mark COMPLETE in WAVES.org

Can run in PARALLEL with Prompts 4 (Demos) and 5 (Subdomain).
```

### Wave 4 Session

```
Working on: Wave 4, Prompt 7 - Landing page with subdomain checker

Steps:
1. Check WAVES.org: Verify ALL of Wave 3 is COMPLETE
2. Read prompt: .claude/prompts/20251204134046-landing-page-with-subdomain-checker-ui.org
3. Update WAVES.org: status=IN PROGRESS
4. Reflect: Dioxus signals, debouncing, modal UI
5. Propose improvements
6. Implement landing.rs
7. Test: subdomain checker, easter eggs, modal
8. Update prompt with discoveries
9. Mark COMPLETE in WAVES.org
10. Unblock Wave 5

Depends on Wave 3 complete.
```

### Wave 5 Session

```
Working on: Wave 5, Prompt 8 - Deploy and test prod infrastructure

Steps:
1. Check WAVES.org: Verify ALL waves 1-4 COMPLETE
2. Read prompt: .claude/prompts/20251204134048-deploy-and-test-production-infrastructure.org
3. Update WAVES.org: status=IN PROGRESS
4. Reflect: Tailscale setup, Terraform apply order, testing plan
5. Propose improvements
6. Execute deployment (carefully!)
7. Run all tests
8. Upload demo images
9. Verify everything works
10. Document any issues
11. Update prompt with discoveries
12. Mark COMPLETE in WAVES.org

FINAL WAVE - launches bits.page to production!
```

## Tips for Success

### For You (Coordinating)

- ✅ Monitor `WAVES.org` for status updates
- ✅ Review completed work before starting next wave
- ✅ Be available to unblock sessions if needed
- ✅ Commit frequently with clear messages

### For Sessions

- ✅ Always read `WAVES.org` first
- ✅ Update files as you go, not at the end
- ✅ Ask questions if assumptions are unclear
- ✅ Document WHY not just WHAT in discoveries
- ✅ Test thoroughly before marking complete

## Quick Start

1. **Now:** Start Wave 1 sessions (both prompts can run in parallel)
2. **After Wave 1:** Start Wave 2 (blocks Wave 3, so do first)
3. **After Wave 2:** Start Wave 3 sessions (3 prompts in parallel)
4. **After Wave 3:** Start Wave 4
5. **After Wave 4:** Start Wave 5 (deployment!)

Estimated total time: ~16 hours (or less with parallelization)
