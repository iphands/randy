for var in `/bin/ls -d /proc/[0-9]*`
do
    if [ "`fgrep -q RSS ${var}/status ; echo $?`" == "1" ]
    then
        cat ${var}/status | fgrep -e Name -e RSS
    fi
done
