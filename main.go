package main

import "fmt"
import "github.com/webview/webview"

func main() {
	fmt.Println("Hello, World")

	debug := true
	w := webview.New(debug)
	defer w.Destroy()
	w.SetTitle("Minimal Webview Example")
	w.SetSize(800, 600, webview.HintNone)
	w.Navigate("https://en.m.wikipedia.org/wiki/Main_Page")
	w.Run()
}
