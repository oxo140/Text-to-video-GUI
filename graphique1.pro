QT       += core gui

greaterThan(QT_MAJOR_VERSION, 4): QT += widgets
RC_FILE += resources.rc

CONFIG += c++17

# You can make your code fail to compile if it uses deprecated APIs.
# In order to do so, uncomment the following line.
#DEFINES += QT_DISABLE_DEPRECATED_BEFORE=0x060000    # disables all the APIs deprecated before Qt 6.0.0

SOURCES += \
    main.cpp \
    mainwindow.cpp

HEADERS += \
    mainwindow.h

FORMS += \
    mainwindow.ui

# Default rules for deployment.
qnx: target.path = /tmp/$${TARGET}/bin
else: unix:!android: target.path = /opt/$${TARGET}/bin
!isEmpty(target.path): INSTALLS += target



win32: LIBS += -L$$PWD/../../../../Qt/OpenCV-MinGW-Build-OpenCV-4.5.5-x64/x64/mingw/lib/ -llibopencv_core455.dll
win32: LIBS += -L$$PWD/../../../../Qt/OpenCV-MinGW-Build-OpenCV-4.5.5-x64/x64/mingw/lib/ -llibopencv_highgui455.dll
win32: LIBS += -L$$PWD/../../../../Qt/OpenCV-MinGW-Build-OpenCV-4.5.5-x64/x64/mingw/lib/ -llibopencv_imgproc455.dll
win32: LIBS += -L$$PWD/../../../../Qt/OpenCV-MinGW-Build-OpenCV-4.5.5-x64/x64/mingw/lib/ -llibopencv_video455.dll
win32: LIBS += -L$$PWD/../../../../Qt/OpenCV-MinGW-Build-OpenCV-4.5.5-x64/x64/mingw/lib/ -llibopencv_videoio455.dll

INCLUDEPATH += $$PWD/../../../../Qt/OpenCV-MinGW-Build-OpenCV-4.5.5-x64/include
DEPENDPATH += $$PWD/../../../../Qt/OpenCV-MinGW-Build-OpenCV-4.5.5-x64/include

DISTFILES += \
    resources.rc

