In order to prove the specifications, run 
           ```certoraRun contracts/Bridge.sol:Bridge --verify Bridge:specs/Bridge.spec --wait_for_results --rule *rule_name*```
            from the ```solidity``` directory. If you do not have the Certora prover, a link to the results is included inside the .spec files for each rule.


MOVE: * Unit tests written directly in the contract files. To run the tests, from within the ```move``` folder, run ```iota move test```