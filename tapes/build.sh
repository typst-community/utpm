APWD=`pwd`
cd tapes
PRUN=`pwd`
for e in `ls tapes | grep ".*[^\.sh]$" `
do
    if [[ -f tapes/$e/run.sh ]]; then
        cd $e
        bash tapes/$e/run.sh  
        cd $PRUN
    fi
done
cd $APWD