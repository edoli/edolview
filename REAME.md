JDK version used for compiling should be same as bundled jre version.

# Ubuntu compile guide

## Dependencies

* Java (Recommend [Azul Java](https://www.azul.com/downloads/?package=jdk))
* [OpenCV](https://opencv.org/releases/)
* apt install
```
sudo apt update
sudo apt install -y cmake build-essential ant
```

## Compile OpenCV

Set environment variables

```
export JAVA_HOME={java_path}
export ANT_DIR=/usr
```

Go to opencv directory
```
mkdir build
cd build
cmake -DBUILD_LIST=core,imgproc,imgcodecs,java,java_binding_generator ..
make -j8
sudo make install
```

## Compile code
```
export JAVA_HOME={java_path}

./gradlew installDist
```