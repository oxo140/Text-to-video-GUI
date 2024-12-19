#include "mainwindow.h"
#include "./ui_mainwindow.h"
#include "opencv2/opencv.hpp"
#include "iostream"
#include <string>
#include <QString>
#include <QDebug>

using namespace std;
using namespace cv;

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::MainWindow)
{
    ui->setupUi(this);
}

MainWindow::~MainWindow()
{
    delete ui;
}


void MainWindow::on_pushButton_clicked()
{
    QString text = ui->textEdit->toPlainText() ; // text pour la video

    string texte = text.toStdString(); // conversion du text qstring en string

    int vitesseDefilement;
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


      int espaceEntreTexte = 12;
      int epaisseurPolice = 6;


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

      int fontFace = FONT_HERSHEY_TRIPLEX;
      double fontScale = 2.5;



      int largeurImage = 1280; //définiton horizontal ←→
      int hauteurImage = 180;    //définition vertical ↑↓

      largeurImage = ui->textEdit_2->toPlainText().toInt();
      hauteurImage = ui->textEdit_3->toPlainText().toInt();

      int positionX = largeurImage;
      int fps = 60; // Images par seconde

      QString exten = ui->comboBox_3->currentText();


      string extensionvideo = "." + exten.toStdString(); // format du ficher de sortie


      QString nomdufich = ui->textEdit_4->toPlainText() ; // nom du ficher de sortie

      string nomdufichier = nomdufich.toStdString(); // conversion du text qstring en string

      // création de la video



      nomdufichier.append(extensionvideo);

      VideoWriter video( nomdufichier, VideoWriter::fourcc('H', '2', '6', '4'), fps, Size(largeurImage, hauteurImage));

      if (!video.isOpened())
      {
          cout << "Impossible de créer la vidéo." << endl;
      }


      Mat image(hauteurImage, largeurImage, CV_8UC3, Scalar(0, 0, 0)); // couleur du fond


      int texteLargeur = getTextSize(texte, fontFace, fontScale, epaisseurPolice, nullptr).width;
      int texteHauteur = getTextSize(texte, fontFace, fontScale, epaisseurPolice, nullptr).height;

      int positionY = (hauteurImage + texteHauteur) / 2;

      bool texteVisible = true;

      while (texteVisible) // Algoritme pour que la longeur de la video face 1 defillement de droite vers la gauche.
      {
          image.setTo(Scalar(255, 255, 255)); // Effacer l'image précédente

          putText(image, texte, Point(positionX, positionY), fontFace, fontScale, couleurTexte, epaisseurPolice);

          video.write(image);

          positionX -= vitesseDefilement;

          int texteLargeur = getTextSize(texte, fontFace, fontScale, epaisseurPolice, nullptr).width;
          if (positionX < -texteLargeur)
          {
              texteVisible = false;
          }

          if (waitKey(1) == 27)
              break;
      }

      video.release();
}





