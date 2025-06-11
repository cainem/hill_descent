---
trigger: always_on
---

The code is to be written in rust.
The code will be written in a windows environment.

- Always prefer simple solutions
- Avoid duplication of code whenever possible, which means checking for other areas of the codebase that might already have similar code and functionality before adding a new function or struct.
- You are careful to only make changes that are requested or you are confident are well understood and related to the change being requested
- When fixing an issue or bug, do not introduce a new pattern or technology without first exhausting all options for the existing implementation. And if you finally do this, make sure to remove the old implementation afterwards so we don't have duplicate logic.
- Keep the codebase very clean and organized
- Structs should be created in their own files where the file name is the struct name.
- if structs or functions within them become large > 40 lines, then split the struct off into its own mod and have separate files for big functions.
- do not use public fields in structs, use getters and setters instead.
- Avoid writing scripts in files if possible, especially if the script is likely only to be run once
- Avoid having files over 40-100 lines of code (excluding tests). Refactor at that point.
- Mocking data is only needed for tests
- Never overwrite my .env file without first asking and confirming
- Each function written should be fully unit tested with the tests existing in a
- Tested means that it is to have full statement, branch and condition coverage. This only needs to true in the function to which they relate, full coverage of all code in called functions is not required (this will be assumed to be tested elsewhere)
- There should be as little mocking used as possible. There is no I/O so the only mocking required is for the PRNG
- The test names should clearly spell out the desired behavior and results, they should be in the form given_xxx_when_yyy_then_zzz where xxx are the precoditions, yyy are any conditions and zzz are the results
- Every effort should be taken to make sure unit tests run in a timely manner whilst still providing the coverage
- All units tests must be passing before committing to git.