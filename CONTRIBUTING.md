# Contributing to BRRO Compressor

Thank you for considering contributing to the BRRO Compressor project! Whether it's a bug report, new feature, correction, or additional documentation, we greatly value and appreciate your efforts to improve this project.

Below are the guidelines for contributing to the BRRO Compressor. Please take a moment to review them before submitting your contributions.

## Table of Contents
- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
    - [Reporting Issues](#reporting-issues)
    - [Submitting Code Changes](#submitting-code-changes)
    - [Coding Standards](#coding-standards)
- [Running Tests](#running-tests)
- [Documentation](#documentation)

## Code of Conduct

This project adheres to a code of conduct that is based on respect, collaboration, and inclusivity. By participating in this project, you agree to uphold these values. Please report any issues or behavior that violates these guidelines to the project maintainers.

## How to Contribute

### Reporting Issues

If you encounter a bug, performance issue, or have a suggestion for an enhancement, please open an issue in our [GitHub repository](https://github.com/instaclustr/fft-compression/issues). Make sure to provide as much detail as possible to help us understand the problem or suggestion. **But check for existing issues first!**

When reporting an issue, please include:
- A clear and descriptive title.
- A detailed description of the issue or suggestion.
- Steps to reproduce the issue, if applicable.
- Any relevant logs, screenshots, or output that might help us diagnose the problem.

### Submitting Code Changes

We accept contributions via Pull Requests (PRs). Before submitting a PR, please follow these steps:

1. **Fork the repository** to your GitHub account.
2. **Create a new branch** for your changes.
   - If you're creating a bugfix branch, name it XXX-something, where XXX is the issue number.
   - If you're working on a new feature, first create an issue to announce your intentions, then name the branch XXX-something, where XXX is the issue number.
3. **Make your changes** in this branch.
4. **Test your changes** to ensure that they do not break existing functionality.
5. **Commit your changes** with a meaningful commit message.
6. **Push your changes** to your forked repository.
7. **Create a Pull Request** to the main branch of the original repository.

#### How to Create a Pull Request (PR)

1. Go to the [GitHub repository](https://github.com/instaclustr/fft-compression).
2. Click on the "Pull Requests" tab.
3. Press the **"New pull request"** button.
4. Select the **base repository** and the branch where you want to merge your changes (`main`).
5. Compare this with the branch containing your changes (your forked repository and branch).
6. Provide a **descriptive title** and **clear description** of the changes in the PR.
7. Link any relevant issues by adding `Closes #IssueNumber` to the description.
8. Submit your pull request.

#### Guidelines for Pull Requests

- Provide a clear description of your changes and the problem they solve.
- Reference any relevant issues or discussions in your PR description.
- When you make a PR for small change (such as fixing a typo, grammar fix), please squash your commits so that we can maintain a cleaner git history.
- Include any necessary documentation updates as part of your PR.
- Ensure that your code adheres to the project's coding standards (see below).
- Code review comments might be added to your pull request. Please discuss them, implement the suggested changes, and push additional commits to your feature branch. Make sure to leave a comment after pushing. 
The new commits will appear in the pull request automatically, but the reviewers won't be notified unless you comment.
- If your PR is a work-in-progress (WIP), please indicate this in the title by prefixing it with `[WIP]`.

If your pull request isn't accepted right away, don't get discouraged! If there are issues with the implementation, you'll likely receive feedback on what can be improved.

### Coding Standards

To maintain consistency and quality across the codebase, please adhere to the following coding standards:

- Follow the naming conventions and coding style already present in the codebase.
- Keep functions and methods small and focused.
- Write descriptive comments where necessary, especially in complex sections of code.
- Ensure your code is linted and formatted according to the project's style guides.

## Running Tests

All code changes should be covered by tests. To run the test suite, use the following command: `cargo test`

## Documentation

Good documentation is crucial for the usability and maintainability of the project. If you are adding a new feature or modifying existing behavior, ensure that you update the relevant documentation files.

- See `docs/README.md` for more information on building the docs and how docs get released.
- Ensure that your changes are reflected in the usage instructions or other relevant sections.

## Roadmap

We have a long-term vision for the BRRO Compressor project, and we encourage contributors to get involved with features that are planned for future releases. Below is a brief overview of our current roadmap:

- Implement streaming compression/decompression.
- Expand frames to allow new data to be appended to existing frames.

Feel free to contribute to these ongoing efforts or propose your own enhancements!

## Feedback and Suggestions

We value your feedback and suggestions! If you have any ideas on how to improve the BRRO Compressor, please don't hesitate to share them. You can do so by:

- Opening a new issue labeled as a `suggestion` in our [GitHub repository](https://github.com/instaclustr/fft-compression).

Your input helps us shape the future of the project, and we appreciate any suggestions or feedback you can provide.
