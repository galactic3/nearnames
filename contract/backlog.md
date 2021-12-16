rewrite prefix handling using option: https://github.com/near/near-sdk-rs/blob/master/HELP.md#generating-unique-prefixes-for-persistent-collections
use PanicOnDefault https://github.com/near/near-sdk-rs/blob/master/HELP.md#use-panicondefault
ability to sell same contract multiple times
clean up imports, make them rust idiomatic
reduce rust visibilities as much as possible
document and refine cross contract promises flows
replace U128 conversions with .0
clear warnings
refactor context builder methods
ensure every method has at least one positive and negative case
standartize bash scripts, use ones from sdk examples if possible
profile gas and storage limits
check borrowing is used everywhere possible
define sources of truth for frontend
fix tests not linked to actual code
how to check balance in unit tests
make duration string serialized
think through contract structures upgrades
clarify if it's ok that lot disappears after successful sale
apply oop to lot, profile
write checklist and put in readme
security audit
improve scalability
improve e2e test coverage
provile gas usages
improve tokenomics
Allow lot to call claim on itself ( to clean up state and be able to resell )
Adopt single strategy for ref vs value arg passing
make fraction dumb again
use standard to_yocto
