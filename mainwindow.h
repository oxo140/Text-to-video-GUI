#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>

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

private:
    Ui::MainWindow *ui;
};
#endif // MAINWINDOW_H
