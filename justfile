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


[no-cd]
new-day day:
    cargo init --bin {{day}}
    cp ../template-main.rs {{day}}/src/main.rs
    touch {{day}}/test_input.txt
