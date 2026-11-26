#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include <QMainWindow>
#include <QtCore>
#include <QtWidgets>
#include <QtGui>


QT_BEGIN_NAMESPACE
namespace Ui { class MainWindow; }
QT_END_NAMESPACE

class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    MainWindow(QWidget *parent = nullptr);
    ~MainWindow();

private slots:
    void on_pushButton_clicked();

    void on_comboBox_activated(int index);

    void on_comboBox_2_activated(int index);

    void on_textEdit_2_copyAvailable(bool b);

    void on_textEdit_3_copyAvailable(bool b);

    void on_textEdit_4_copyAvailable(bool b);

    void on_comboBox_3_activated(int index);

    void on_pushButton_2_clicked();

    void on_pushButton_3_clicked();

    void on_pushButton_4_clicked();

    void on_pushButton_5_clicked();

    void on_textEdit_textChanged();

    void on_textEdit_4_textChanged();

private:
    Ui::MainWindow *ui;
};
#endif // MAINWINDOW_H
