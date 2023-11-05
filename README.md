<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->
<a name="readme-top"></a>

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]



<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/kingwingfly/downloader">
    <img src="screenshots/app-icon.png" alt="Logo" width="80" height="80">
  </a>

<h3 align="center">downloader</h3>

  <p align="center">
    A downloader in Rust, Tauri, nextjs and actix, easy to be extended.
    <br />
    <a href="https://github.com/kingwingfly/downloader"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/kingwingfly/downloader">View Demo</a>
    ·
    <a href="https://github.com/kingwingfly/downloader/issues">Report Bug</a>
    ·
    <a href="https://github.com/kingwingfly/downloader/issues">Request Feature</a>
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project

[![Product Name Screen Shot][product-screenshot]](https://example.com)

This is a downloader, I have implemented bilibili download now. This downloader is easy to secondary development, you just need impl parser for the website you want to download in parser.rs.

The config or the cookies you store in this app will be encrypted by `rsa`, and the private key will be store using `keyring` in your system's key keeper (such as key-chain on macOS).

### Important

This app uses ffmpeg to merge the video and audio, so you should add ffmpeg to env path or config it in config page. On macOS, you should config full path of `ffmpeg`, for binary on macOS can not be invoked directly by name. 

![config ffmpeg](/screenshots/config_ffmpeg.png)



<p align="right">(<a href="#readme-top">back to top</a>)</p>



### Built With

* rust
* tauri
* [![Next][Next.js]][Next-url]
* [![React][React.js]][React-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- GETTING STARTED -->
## Getting Started

**This app used `async trait` in rust, so you need a `nightly toolchain` installed.**

dev
```
cargo tauri dev
```
build yourself
```
cargo tauri build
```

### Prerequisites

* npm
  ```sh
  npm install npm@latest -g
  ```

### Installation

1. Clone the repo
    ```sh
    git clone https://github.com/kingwingfly/downloader.git
    ```
2. Install NPM packages
    ```sh
    npm install
    ```
3. Install `tauri-cli` and `rust nightly toolchain`
    ```sh
    cargo install tauri-cli
    rustup install nightly
    ```
4. Build the bundle
    ```sh
    cargo tauri build
    ```

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage

New a Task

![new](/screenshots/newTask.png)

See the process

![list](/screenshots/taskList.png)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ROADMAP -->
## Roadmap

- [ ] Fix: tauri build goes wrong with nextjs' parellel route
- [ ] New: more website support

See the [open issues](https://github.com/kingwingfly/downloader/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- LICENSE -->
## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTACT -->
## Contact

Louis - 20200581@cqu.edu.cn

Project Link: [https://github.com/kingwingfly/downloader](https://github.com/kingwingfly/downloader)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

* [rust](https://www.rust-lang.org)
* [tauri](https://tauri.app)
* [nextjs](https://nextjs.org)
* [react](https://react.dev)
* [tailwind css](https://tailwindcss.com)
* [actix](https://actix.rs/docs/actix)
* [snafu](https://docs.rs/snafu/latest/snafu/index.html)
* [reqwest](https://docs.rs/reqwest/latest/reqwest/index.html)
* [keyring](https://docs.rs/keyring/2.0.5/keyring/index.html)
* [tracing](https://docs.rs/tracing/latest/tracing/index.html)
* [tempdir](https://docs.rs/temp-dir/0.1.11/temp_dir/index.html)
* [scraper](https://docs.rs/scraper/latest/scraper/index.html)
* [rsa](https://docs.rs/rsa/0.9.3/rsa/index.html)




<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/kingwingfly/downloader.svg?style=for-the-badge
[contributors-url]: https://github.com/kingwingfly/downloader/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/kingwingfly/downloader.svg?style=for-the-badge
[forks-url]: https://github.com/kingwingfly/downloader/network/members
[stars-shield]: https://img.shields.io/github/stars/kingwingfly/downloader.svg?style=for-the-badge
[stars-url]: https://github.com/kingwingfly/downloader/stargazers
[issues-shield]: https://img.shields.io/github/issues/kingwingfly/downloader.svg?style=for-the-badge
[issues-url]: https://github.com/kingwingfly/downloader/issues
[license-shield]: https://img.shields.io/github/license/kingwingfly/downloader.svg?style=for-the-badge
[license-url]: https://github.com/kingwingfly/downloader/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/linkedin_username
[product-screenshot]: screenshots/home.png
[Next.js]: https://img.shields.io/badge/next.js-000000?style=for-the-badge&logo=nextdotjs&logoColor=white
[Next-url]: https://nextjs.org/
[React.js]: https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB
[React-url]: https://reactjs.org/
[Vue.js]: https://img.shields.io/badge/Vue.js-35495E?style=for-the-badge&logo=vuedotjs&logoColor=4FC08D
[Vue-url]: https://vuejs.org/
