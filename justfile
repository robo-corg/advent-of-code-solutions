default:
    echo 'Hello, world!'

[no-cd]
run-test-input:
    cat test_input.txt | cargo run

[no-cd]
run-input:
    cat input.txt | cargo run

[no-cd]
fetch-problem:
    aoc d --input-file=input.txt