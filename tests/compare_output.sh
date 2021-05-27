cargo test integration
test_dirs=$(ls -1 ./output/ | grep test_)
if [ "$test_dirs" = '' ]; then
    echo All tests passed! Exiting.
    exit
else
    echo Melding outputs of failed integration tests
    for test_dir in $test_dirs
    do
        expected_file_name=$(echo $test_dir | sed -e 's/_0.3.0/.rs/')
        meld ./tests/data/expected/$expected_file_name ./output/$test_dir/rpc_response_types.rs 
    done
fi
