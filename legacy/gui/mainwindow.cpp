#include "mainwindow.h"
#include "./ui_mainwindow.h"
#include <opencv2/video/video.hpp>
#include <opencv2/videoio/videoio.hpp>
#include <opencv2/highgui/highgui.hpp>
#include <opencv2/core/core.hpp>
#include <opencv2/imgproc/imgproc.hpp>
#include "iostream"
#include <string>
#include <QString>
#include <QDebug>
#include <QDir>
#include <QFileInfo>
#include <QDesktopServices>
#include <QUrl>

using namespace std;
using namespace cv;

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::MainWindow)
{
    ui->setupUi(this);
    ui->label_7->hide();
    ui->label_8->hide();
    ui->pushButton_2->hide();
    ui->pushButton_4->hide();


}

MainWindow::~MainWindow()
{
    delete ui;
}


void MainWindow::on_pushButton_clicked()
{


    if (ui->textEdit->toPlainText().trimmed().isEmpty()) {
        // Met le QTextEdit en rouge
        ui->textEdit->setStyleSheet("background-color: red;");

        // Affiche un message d'erreur
        QMessageBox::critical(this, "Erreur", "Le champ de texte est vide.");
        if (ui->textEdit_4->toPlainText().trimmed().isEmpty()) {
            // Met le QTextEdit en rouge
            ui->textEdit_4->setStyleSheet("background-color: red;");

            // Affiche un message d'erreur
            QMessageBox::critical(this, "Erreur", "Le champ de nom du fichier est vide.");
            return;

        } else {
            // Réinitialise le style du QTextEdit
            ui->textEdit_4->setStyleSheet("");


        } // champ de nom du fichier vide
        return;

    } else {
        // Réinitialise le style du QTextEdit
        ui->textEdit->setStyleSheet("");


    } //champ de texte vide

    if (ui->textEdit_4->toPlainText().trimmed().isEmpty()) {
        // Met le QTextEdit en rouge
        ui->textEdit_4->setStyleSheet("background-color: red;");

        // Affiche un message d'erreur
        QMessageBox::critical(this, "Erreur", "Le champ de nom du fichier est vide.");
        return;

    } else {
        // Réinitialise le style du QTextEdit
        ui->textEdit_4->setStyleSheet("");


    } // champ de nom du fichier vide


    ui->textEdit->setStyleSheet("");

    ui->textEdit_4->setStyleSheet("");
    // Chemin du répertoire
    QString dirPath = "C:/textvideo";


    // Vérifie si le répertoire existe
    QFileInfo dirInfo(dirPath);

    if (!dirInfo.exists() || !dirInfo.isDir()) {
        // Le répertoire n'existe pas, on le crée
        QDir dir;
        if (dir.mkpath(dirPath)) {
            qDebug() << "Répertoire créé avec succès : " << dirPath;
        } else {
            qDebug() << "Échec de la création du répertoire : " << dirPath;
        }
    } else {
        qDebug() << "Le répertoire existe déjà : " << dirPath;
    }


    QString text = ui->textEdit->toPlainText();
    string texte = text.toStdString();
    ui->label_7->hide();

    int vitesseDefilement=0;
    int introuge =0;
    int intvert =0;
    int intbleu =0;

    if(ui->comboBox->currentText() == "Lent")
    {
        vitesseDefilement = 4;
    }
    else if (ui->comboBox->currentText() == "Normal")
    {
        vitesseDefilement = 8;
    }
    else if (ui->comboBox->currentText() == "Rapide")
    {
        vitesseDefilement = 12;
    }

    if(ui->comboBox_2->currentText() == "Rouge")
    {
        introuge = 255;
    }
    if(ui->comboBox_2->currentText() == "Vert")
    {
        intvert = 255;
    }
    if(ui->comboBox_2->currentText() == "Bleu")
    {
        intbleu = 255;
    }

    Scalar couleurTexte(intbleu, intvert, introuge); // couleur du text noir


    int largeurImage = ui->textEdit_2->toPlainText().toInt();
    int hauteurImage = ui->textEdit_3->toPlainText().toInt();
    if (largeurImage <= 0 || hauteurImage <= 0) {
        std::cout << "Dimensions invalides !" << std::endl;
        return;
    }

    int fps = 60;
    QString nomdufich = ui->textEdit_4->toPlainText();

    string nomdufichierWebM = "C:\\textvideo\\" + nomdufich.toStdString() + ".webm";
    string nomdufichier = "C:\\textvideo\\" + nomdufich.toStdString() + ".avi";

    std::cout << "Nom du fichier : " << nomdufichier << std::endl;

    VideoWriter video(nomdufichier, VideoWriter::fourcc('M', 'J', 'P', 'G'), fps, Size(largeurImage, hauteurImage));
    if (!video.isOpened()) {

        std::cout << "Impossible de créer la vidéo." << std::endl;
        ui->label_8->show();

        QMessageBox::critical(this, "Erreur", "Vérifier si FFMPEG est installé.");

        return;

        }


    Mat image(hauteurImage, largeurImage, CV_8UC3, Scalar(255, 255, 255));

    int texteLargeur = getTextSize(texte, FONT_HERSHEY_TRIPLEX, 2.5, 6, nullptr).width;
    int texteHauteur = getTextSize(texte, FONT_HERSHEY_TRIPLEX, 2.5, 6, nullptr).height;
    int positionX = largeurImage, positionY = (hauteurImage + texteHauteur) / 2;

    std::cout << "Largeur texte: " << texteLargeur << ", Hauteur texte: " << texteHauteur << std::endl;
    std::cout << "Position texte X: " << positionX << ", Position texte Y: " << positionY << std::endl;

    while (positionX + texteLargeur > 0) {
        image.setTo(Scalar(255, 255, 255)); // Réinitialiser l'image à chaque itération
        putText(image, texte, Point(positionX, positionY), FONT_HERSHEY_TRIPLEX, 2.5, couleurTexte, 6); // Ajouter le texte
        video.write(image); // Ajouter l'image à la vidéo
        positionX -= vitesseDefilement; // Déplacer le texte
    }

    // Ajouter 1 seconde de vidéo après le défilement du texte
    int frameCount = fps; // Nombre de frames à ajouter (1 seconde à 60 fps)
    for (int i = 0; i < frameCount; ++i) {
        image.setTo(Scalar(255, 255, 255)); // Réinitialiser l'image (fond blanc)
        video.write(image); // Ajouter l'image fixe à la vidéo

        ui->label_7->hide();
    }

    video.release();
    std::cout << "Vidéo créée avec succès !" << std::endl;


    std::string commandeFFmpeg = "ffmpeg -y -i \"" + nomdufichier + "\" -c:v libvpx-vp9 -b:v 2M \"" + nomdufichierWebM + "\"";
    std::cout << "Conversion en WebM..." << std::endl;
    int resultat = system(commandeFFmpeg.c_str());

    if (resultat == 0) {
        std::cout << "Conversion réussie ! Fichier WebM : " << nomdufichierWebM << std::endl;

        // Suppression du fichier AVI
        try {
            if (std::filesystem::remove(nomdufichier)) {
                std::cout << "Fichier AVI supprimé : " << nomdufichier << std::endl;
            } else {
                std::cout << "Impossible de supprimer le fichier AVI." << std::endl;
                QMessageBox::critical(this, "Erreur", "Vérifier si FFMPEG est installé.");

                return;
            }
        } catch (const std::filesystem::filesystem_error& e) {
            std::cout << "Erreur lors de la suppression du fichier AVI : " << e.what() << std::endl;
            QMessageBox::critical(this, "Erreur", "Vérifier si FFMPEG est installé.");

            return;
        }
    } else {
        std::cout << "Erreur lors de la conversion en WebM. Le fichier AVI n'a pas été supprimé." << std::endl;
        QMessageBox::critical(this, "Erreur", "Vérifier si FFMPEG est installé.");

        return;
    }
    ui->pushButton_2->show();
    ui->pushButton_4->show();
    ui->label_7->show();
    ui->label_8->hide();
    qDebug() << text;
    qDebug() << texte;



}






void MainWindow::on_comboBox_2_activated(int index)
{

}


void MainWindow::on_comboBox_3_activated(int index)
{

}


void MainWindow::on_textEdit_2_copyAvailable(bool b)
{

}


void MainWindow::on_textEdit_3_copyAvailable(bool b)
{

}


void MainWindow::on_textEdit_4_copyAvailable(bool b)
{

}


void MainWindow::on_comboBox_activated(int index)
{

}


void MainWindow::on_pushButton_2_clicked()
{


        QString dirPath = "C:/textvideo";

        // Créer un objet QUrl avec le chemin du répertoire
        QUrl url = QUrl::fromLocalFile(dirPath);

        // Ouvrir le répertoire dans l'explorateur de fichiers
        if (QDesktopServices::openUrl(url)) {
            qDebug() << "Le répertoire a été ouvert avec succès : " << dirPath;
        } else {
            qDebug() << "Échec de l'ouverture du répertoire : " << dirPath;


        }





}


void MainWindow::on_pushButton_3_clicked()
{

}


void MainWindow::on_pushButton_4_clicked()
{
    QString nomdufich = ui->textEdit_4->toPlainText();

    // Construire le chemin complet du fichier avec l'extension .webm
    QString nomdufichierWebM = "C:\\textvideo\\" + nomdufich + ".webm";

    // Vérifier si le fichier existe avant d'essayer de l'ouvrir
    if (QFile::exists(nomdufichierWebM)) {
        // Utiliser QProcess pour ouvrir le fichier avec l'application par défaut
        QProcess::startDetached("explorer", QStringList() << nomdufichierWebM);
    } else {
        qDebug() << "Le fichier n'existe pas à l'emplacement spécifié : " << nomdufichierWebM;
    }
}


void MainWindow::on_pushButton_5_clicked()
{




}


void MainWindow::on_textEdit_textChanged()
{
    ui->pushButton_2->hide();
    ui->pushButton_4->hide();
    ui->label_7->hide();
}


void MainWindow::on_textEdit_4_textChanged()
{
    ui->pushButton_2->hide();
    ui->pushButton_4->hide();
    ui->label_7->hide();

}

