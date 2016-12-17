#!/bin/dash

date=`date +%Y%0m01`
lang='en'

base=https://dumps.wikimedia.org/enwiki/$date/
#                                           Compressed  Uncompressed
redr=$lang'wiki-'$date'-redirect.sql.gz'    # 107.6 MB
lnks=$lang'wiki-'$date'-pagelinks.sql.gz'   # 4.9 GB
page=$lang'wiki-'$date'-page.sql.gz'        # 1.4 GB

#redr_url=$base$redr 
#lnks_url=$base$lnks
#page_url=$base$page

wget $base$redr 2> /dev/null && gunzip $redr &
wget $base$lnks 2> /dev/null && gunzip $lnks &
wget $base$page 2> /dev/null && gunzip $page &

wait
