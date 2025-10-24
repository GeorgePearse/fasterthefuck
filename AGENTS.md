# FastertTheFuck - Agent Instructions

## Critical Guidelines for All Agents

### Time Estimates Are Forbidden
**NEVER provide time estimates.** They are always stupidly pessimistic and create false expectations. Development velocity varies wildly based on:
- Complexity of specific rule implementations
- Unexpected architectural decisions needed
- Integration challenges with existing code
- Test coverage and edge case handling
- Debugging sessions for subtle bugs

Instead of estimates, focus on:
- Breaking work into discrete, shippable chunks
- Tracking actual progress via commits
- Describing what will be done and in what order
- Letting velocity emerge naturally through execution

### Code Quality Standards

1. **Fully Typed Rust Code**
   - Use `cargo check` and `zuban` for rapid type feedback
   - Leverage Rust's type system to catch errors early
   - No unwrap() without explicit documentation

2. **Comprehensive Testing**
   - Unit tests for all new modules
   - Integration tests for cross-module interaction
   - Property-based tests where applicable
   - Run tests before commits: `cargo test --lib`

3. **Clean Git History**
   - Commit after each meaningful milestone
   - Meaningful commit messages describing "why" not "what"
   - Use `git add -A && git commit` after significant changes
   - Pre-commit hooks should pass

### Rule Implementation Strategy

**Use SimpleRuleBuilder for Quick Porting**
- Most simple rules can be ported in <50 lines of Rust
- Use the builder pattern to avoid boilerplate
- Test each rule with at least 2 test cases

**Pattern-Based Organization**
- Group related rules in category modules (git/, permissions/, etc.)
- Share common matching logic where possible
- Leverage macros and builders to reduce duplication

**Iterative Porting**
- Port in waves by category, not randomly
- Each wave is independently testable
- Can ship working rules incrementally

### Development Workflow

1. **Read Plan Mode Instructions**
   - Never edit code without explicit user approval in plan mode
   - Present clear plans before implementation
   - Ask clarifying questions if requirements are ambiguous

2. **Use TodoWrite Proactively**
   - Track all significant tasks
   - Mark tasks as in_progress and completed immediately
   - Update descriptions with actual progress

3. **Leverage Specialized Agents**
   - Use Explore agent for codebase searches
   - Use consensus for architectural decisions
   - Use thinkdeep for complex algorithm design
   - Use debugger for tricky bugs

4. **Commit Strategy**
   - Commit after each phase/section completion
   - Run `cargo test --lib` before every commit
   - Push to origin regularly
   - Keep commits focused and reviewable

### Rule Categories & Complexity

**Simple Rules (SimpleRuleBuilder)**
- ~40% of rules: Basic string matching/replacement
- Example: `git_branch_delete` (-d → -D)
- Time to implement: <10 minutes per rule

**Regex Rules (Planned RegexRuleBuilder)**
- ~25% of rules: Pattern matching with captures
- Example: `git_push` (extract branch name, suggest upstream)
- Requires regex knowledge but still straightforward

**Fuzzy Rules (Planned FuzzyRuleBuilder)**
- ~15% of rules: Fuzzy matching against known values
- Example: `cd_correction` (suggest closest directory)
- Uses existing FuzzyMatcher in core

**Complex Rules (Custom Implementation)**
- ~20% of rules: Multi-step logic, filesystem ops, external commands
- Example: `cd_mkdir` (try cd, fallback to mkdir)
- Requires careful design and edge case handling

### Performance Targets

- Sub-100ms average correction time (vs ~500ms Python)
- Parallel rule evaluation via Rayon (non-negotiable)
- Lazy compilation of regex patterns
- Rule caching for frequently-matched patterns

### When to Ask for Help

- **Architecture questions**: Use consensus for multi-model review
- **Complex bugs**: Use debugger agent for systematic investigation
- **Codebase navigation**: Use Explore agent instead of grep
- **Algorithm design**: Use thinkdeep for deep analysis
- **Code quality**: Use code-reviewer after significant changes

### Success Criteria

✅ Code compiles without warnings
✅ All tests pass: `cargo test --lib`
✅ Meaningful git history with clear commits
✅ Progress visible through github.com/GeorgePearse/fasterthefuck
✅ Rules are production-ready, not half-baked
✅ Architecture remains clean and maintainable

### Anti-Patterns to Avoid

❌ Time estimates (banned!)
❌ Partial implementations ("TODO: finish later")
❌ Tests that don't actually test anything
❌ Commits with mixed concerns (bundle related changes)
❌ Code comments that state the obvious
❌ Copy-paste rule implementations (use builders/macros instead)
❌ Ignoring compiler warnings
