#!/bin/bash

if [ -z "$BASH_VERSION" ]
then
    exec bash "$0" "$@"
fi

if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root" 
   exit 1
fi


echo Install opencv dependencies

cp lib/libopencv_core.so.4.5 /usr/local/lib
cp lib/libopencv_imgproc.so.4.5 /usr/local/lib
cp lib/libopencv_imgcodecs.so.4.5 /usr/local/lib
cp lib/libopencv_java4.so /usr/lib64


echo Install application

rm -rf /usr/lib/edolview
cp -r ./ /usr/lib/edolview
rm /usr/lib/edolview/install.sh
rm -f /usr/bin/edolview
ln -s /usr/lib/edolview/bin/desktop /usr/bin/edolview

echo Create desktop file

APP_HOME=`pwd -P`

rm -f /usr/share/applications/edolview.desktop 
cat >/usr/share/applications/edolview.desktop <<EOL
[Desktop Entry]
Comment=Image viewer
Name=Edolview
Exec=edolview
Terminal=false
Type=Application
Icon=/usr/lib/edolview/icon.png
StartupWMClass=Edolview
Categories=Graphics;
EOL

echo Installation finished