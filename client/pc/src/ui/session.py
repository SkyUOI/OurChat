# Form implementation generated from reading ui file '.\session.ui'
#
# Created by: PyQt6 UI code generator 6.4.2
#
# WARNING: Any manual changes made to this file will be lost when pyuic6 is
# run again.  Do not edit this file unless you know what you are doing.


from PyQt6 import QtCore, QtWidgets


class Ui_Session(object):
    def setupUi(self, Session):
        Session.setObjectName("Session")
        Session.resize(400, 300)
        Session.setWindowTitle("Form")
        self.horizontalLayout = QtWidgets.QHBoxLayout(Session)
        self.horizontalLayout.setObjectName("horizontalLayout")
        self.left_panel = QtWidgets.QVBoxLayout()
        self.left_panel.setObjectName("left_panel")
        self.horizontalLayout_4 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_4.setObjectName("horizontalLayout_4")
        self.search_box = QtWidgets.QLineEdit(parent=Session)
        self.search_box.setObjectName("search_box")
        self.horizontalLayout_4.addWidget(self.search_box)
        self.add_session_btn = QtWidgets.QToolButton(parent=Session)
        self.add_session_btn.setMaximumSize(QtCore.QSize(30, 30))
        self.add_session_btn.setText("+")
        self.add_session_btn.setObjectName("add_session_btn")
        self.horizontalLayout_4.addWidget(self.add_session_btn)
        self.left_panel.addLayout(self.horizontalLayout_4)
        self.session_list = QtWidgets.QListWidget(parent=Session)
        self.session_list.setResizeMode(QtWidgets.QListView.ResizeMode.Adjust)
        self.session_list.setObjectName("session_list")
        self.left_panel.addWidget(self.session_list)
        self.horizontalLayout.addLayout(self.left_panel)
        self.right_panel = QtWidgets.QVBoxLayout()
        self.right_panel.setObjectName("right_panel")
        self.horizontalLayout_2 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_2.setObjectName("horizontalLayout_2")
        spacerItem = QtWidgets.QSpacerItem(
            40,
            20,
            QtWidgets.QSizePolicy.Policy.Expanding,
            QtWidgets.QSizePolicy.Policy.Minimum,
        )
        self.horizontalLayout_2.addItem(spacerItem)
        self.title = QtWidgets.QLabel(parent=Session)
        self.title.setText("Title")
        self.title.setObjectName("title")
        self.horizontalLayout_2.addWidget(self.title)
        spacerItem1 = QtWidgets.QSpacerItem(
            40,
            20,
            QtWidgets.QSizePolicy.Policy.Expanding,
            QtWidgets.QSizePolicy.Policy.Minimum,
        )
        self.horizontalLayout_2.addItem(spacerItem1)
        self.right_panel.addLayout(self.horizontalLayout_2)
        self.message_list = QtWidgets.QListWidget(parent=Session)
        self.message_list.setSizeAdjustPolicy(
            QtWidgets.QAbstractScrollArea.SizeAdjustPolicy.AdjustIgnored
        )
        self.message_list.setSelectionMode(
            QtWidgets.QAbstractItemView.SelectionMode.NoSelection
        )
        self.message_list.setVerticalScrollMode(
            QtWidgets.QAbstractItemView.ScrollMode.ScrollPerPixel
        )
        self.message_list.setHorizontalScrollMode(
            QtWidgets.QAbstractItemView.ScrollMode.ScrollPerPixel
        )
        self.message_list.setResizeMode(QtWidgets.QListView.ResizeMode.Adjust)
        self.message_list.setObjectName("message_list")
        self.right_panel.addWidget(self.message_list)
        self.editor = QtWidgets.QTextEdit(parent=Session)
        self.editor.setObjectName("editor")
        self.right_panel.addWidget(self.editor)
        self.horizontalLayout_3 = QtWidgets.QHBoxLayout()
        self.horizontalLayout_3.setObjectName("horizontalLayout_3")
        spacerItem2 = QtWidgets.QSpacerItem(
            40,
            20,
            QtWidgets.QSizePolicy.Policy.Expanding,
            QtWidgets.QSizePolicy.Policy.Minimum,
        )
        self.horizontalLayout_3.addItem(spacerItem2)
        self.send_btn = QtWidgets.QPushButton(parent=Session)
        self.send_btn.setText("send")
        self.send_btn.setObjectName("send_btn")
        self.horizontalLayout_3.addWidget(self.send_btn)
        self.right_panel.addLayout(self.horizontalLayout_3)
        self.right_panel.setStretch(0, 1)
        self.right_panel.setStretch(1, 10)
        self.right_panel.setStretch(2, 6)
        self.right_panel.setStretch(3, 1)
        self.horizontalLayout.addLayout(self.right_panel)
        self.horizontalLayout.setStretch(0, 2)
        self.horizontalLayout.setStretch(1, 5)

        self.retranslateUi(Session)
        QtCore.QMetaObject.connectSlotsByName(Session)

    def retranslateUi(self, Session):
        pass
