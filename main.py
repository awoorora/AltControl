from tkinter import *

from PIL.Image import alpha_composite
from pyautogui import *
import keyboard
import threading
import time

xDict = {}
yDict = {}

for i in range(26):
    xDict[chr(97 + i)] = i
    yDict[chr(97 + i)] = i

print(xDict)
print(yDict)

windowIsOpen = 0

def openWindow():
    global windowIsOpen
    if windowIsOpen:
        return
    
    windowIsOpen = 1
    newWindow = Tk()
    newWindow.geometry('1920x1080+0+0')

    cellWidth = 1920 / 26
    cellHeight = 1080 / 26
    padx = cellWidth * 0.35  # Adjust factor for best spacing
    pady = cellHeight * 0.2
    font_size = int(cellHeight * 0.25)  # Scale font based on cell height

    for i in range(26):
        for j in range(26):
            label = Label(
                newWindow,
                text=chr(97 + i).upper() + chr(97 + j).upper(),
                background="black",
                foreground="lemon chiffon",
                padx=padx,
                pady=pady,
                font=("Consolas", font_size),
            )
            label.grid(row=i, column=j)

    newWindow.protocol("WM_DELETE_WINDOW", lambda: onClose(newWindow))
    newWindow.configure(background='black')
    newWindow.wm_attributes('-transparentcolor', 'black')
    newWindow.state('zoomed')
    newWindow.lift()
    newWindow.overrideredirect(True)
    newWindow.after(1, lambda: newWindow.focus_force())
    newWindow.mainloop()

def onClose(window):
    global windowIsOpen
    window.destroy()
    windowIsOpen = 0

keyBuffer = []

def listenKeys():
    global keyBuffer
    while True:
        event = keyboard.read_event() #checks both presses and releases
        if event.event_type == keyboard.KEY_DOWN:
            keyBuffer.append((event.name, time.time()))
            checkAltCtrl()

def checkAltCtrl():
    global keyBuffer
    bufferTime = 0.5 #buffer time in seconds

    now = time.time()
    keyBuffer = [(key, t) for key, t in keyBuffer if now - t < bufferTime]

    #check if Alt was followed by Ctrl
    for i in range(len(keyBuffer) - 1):
        if keyBuffer[i][0] == 'alt' and keyBuffer[i + 1][0] == 'ctrl':
            print("Thing here!")
            openWindow()
            keyBuffer.clear()

threading.Thread(target=listenKeys).start()
