# Game Development with Rust and WebAssembly
Game Development with Rust and WebAssembly, published by Packt

<a href="https://www.packtpub.com/product/game-development-with-rust-and-webassembly/9781801070973"><img src="https://static.packt-cdn.com/products/9781801070973/cover/smaller" alt="Game Development with Rust and WebAssembly" height="256px" align="right"></a>

This is the code repository for [Game Development with Rust and WebAssembly](https://www.packtpub.com/product/game-development-with-rust-and-webassembly/9781801070973), published by Packt.

**Learn how to run Rust on the web while building a game**

## What is this book about?
The Rust programming language has held the most-loved technology ranking on Stack Overflow for 6 years running, while JavaScript has been the most-used programming language for 9 years straight as it runs on every web browser. Now, thanks to WebAssembly (or Wasm), you can use the language you love on the platform that's everywhere.

This book covers the following exciting features:

* Build and deploy a Rust application to the web using WebAssembly
* Use wasm-bindgen and the Canvas API to draw real-time graphics
* Write a game loop and take keyboard input for dynamic action
* Explore collision detection and create a dynamic character that can jump on and off platforms and fall down holes
* Manage animations using state machines
* Generate levels procedurally for an endless runner
* Load and display sprites and sprite sheets for animations

If you feel this book is for you, get your [copy](https://www.amazon.com/dp/1801070970) today!

<a href="https://www.packtpub.com/?utm_source=github&utm_medium=banner&utm_campaign=GitHubBanner"><img src="https://raw.githubusercontent.com/PacktPublishing/GitHub/master/GitHub.png" 
alt="https://www.packtpub.com/" border="5" /></a>


## Instructions and Navigations

You're currently looking the main branch of this repository, which represents the "completed" state of this book. I say completed because development on this branch is ongoing - specifically the challenges cited in the book are being implemented here. If you want to see the end state of any chapter those are stored as tags, such as https://github.com/PacktPublishing/Game-Development-with-Rust-and-WebAssembly/tree/chapter_1.

**Following is what you need for this book:**

This game development book is for developers interested in Rust who want to create and deploy 2D games to the web. Game developers looking to build a game on the web platform using WebAssembly without C++ programming or web developers who want to explore WebAssembly along with JavaScript web will also find this book useful. The book will also help Rust developers who want to move from the server side to the client side by familiarizing them with the WebAssembly toolchain. Basic knowledge of Rust programming is assumed.

With the following software and hardware list you can run all code files present in the book (Chapter 1-11).

### Software and Hardware List

| Chapter  | Software required                          | version | OS required |
|----------|--------------------------------------------|---------|-------------|
| (1 - 11) | Rust Toolchains via Rustup                 | 1.57.0  | Any OS      |
| (1 - 11) | NodeJS                                     | 16.13.0 | Any OS      |
| (1 - 11) | Rust Compile target wasm32-unknown-unknown | NA      | NA          |

I use https://asdf-vm.com to install Node and a .tool-versions file is present but you don't have to. Instructions for creating a new project are found in the book (chapter 1) but the project can also be setup by cloning this repository and running the commands for building and running. Speaking of that:

### Running this App

#### Installation

`npm install` Will install the Node dependencies (primarily WebPack). Don't worry you don't have to think about those much.

#### Running in debug

`npm start` Will compile the application to Wasm and start a server, running it at localhost:8080 by default. This will also ensure `wasm-pack` is setup and running and run `cargo build`.

#### Building for release

`npm run build` Creates a release build and puts it in the `dist` directory.

#### Running Tests

`npm run test`

You can use a lot of the `cargo` commands as well - but those do not go through the process of bundling up the built assembly for distribution. 

#### Deployment

This branch is setup for continuous deployment with GitHub Actions, as is the tag for chapter_10. Something to keep in mind when forking the repository. The current production version of this game can be found at:

https://rust-games-webassembly.netlify.app

## More Information 
We also provide a PDF file that has color images of the screenshots/diagrams used in this book. [Click here to download it](https://static.packt-cdn.com/downloads/9781801070973_ColorImages.pdf).

The Code in Action videos for this book can be viewed at https://bit.ly/3uxXl4W.

### Related products <Other books you may enjoy>
* Creative Projects for Rust Programmers  [[Packt]](https://www.packtpub.com/product/creative-projects-for-rust-programmers/9781789346220) [[Amazon]](https://www.amazon.com/Creative-Projects-Rust-Programmers-WebAssembly/dp/1789346223)

* Rust Web Programming [[Packt]](https://www.packtpub.com/product/rust-web-programming/9781800560819) [[Amazon]](https://www.amazon.com/Rust-Web-Programming-hands-programming-dp-1800560818/dp/1800560818/ref=mt_other?_encoding=UTF8&me=&qid=)

## Get to Know the Author
**Eric Smith** is a software crafter with over 20 years of software development experience. Since 2005, he's worked at 8th Light, where he consults for companies big and small by delivering software, mentoring developers, and coaching teams. He's a frequent speaker at conferences speaking on topics such as educating developers and test-driven development, and holds a master's degree in video game development from DePaul University. Eric wrote much of the code for this book live on his Twitch stream. When he's not at the computer, you can find Eric running obstacle races and traveling with his family.



