file_path="./tests/data/observed/"
cargo test integration
if [ "$(ls -1 $file_path)" = '' ]; then
    echo All tests passed! Exiting.
else
    echo Melding outputs of failed integration tests
fi
for file_name in `ls -1 $file_path`
do
    #echo $file_path/$file_name
    meld $file_path/$file_name `echo $file_path/$file_name | sed s/observed/expected/` & &>/dev/null
done
