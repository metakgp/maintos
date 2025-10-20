<div id="top"></div>

<!-- PROJECT SHIELDS -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links-->
<div align="center">

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]
[![Wiki][wiki-shield]][wiki-url]

</div>

<!-- PROJECT LOGO -->
<br />
<!-- UPDATE -->
<div align="center">
  <a href="https://github.com/metakgp/maintos">
     <img width="140" alt="image" src="https://raw.githubusercontent.com/metakgp/design/main/logos/black-large.jpg">
  </a>

  <h3 align="center">Maintos</h3>

  <p align="center">
    <i>A maintainers' dashboard for all metaKGP projects deployed on the server.</i>
    <br />
    <a href="https://maintos.metakgp.org">Website</a>
    ·
    <a href="https://github.com/metakgp/maintos/issues">Request Feature / Report Bug</a>
  </p>
</div>


<!-- TABLE OF CONTENTS -->
<details>
<summary>Table of Contents</summary>

- [About The Project](#about-the-project)
- [Development](#development)
- [Deployment](#deployment)
- [Contact](#contact)
  - [Maintainer(s)](#maintainers)
  - [creators(s)](#creators)
- [Additional documentation](#additional-documentation)

</details>


<!-- ABOUT THE PROJECT -->
## About The Project
<div align="center">
  <a href="https://github.com/metakgp/maintos">
    <img width="80%" alt="image" src="https://gist.github.com/user-attachments/assets/e5f7e679-b7d4-413f-a874-2de638461780">
  </a>
</div>

_Maintos_ is a maintainer's dashboard which gives maintainers access to information and control on their projects, without requiring explicit access to the server. Maintainers of a project can start/stop the running containers/services, read logs for the project, as well as see and update environment variables.

<p align="right">(<a href="#top">back to top</a>)</p>

## Development

1. Clone this repository.
2. Backend:
   - Copy `.env.template` to `.env` and update the values as per [Environment Variables](#environment-variables).
   - For the backend to run, Docker must be installed and running.
   - Run the backend:
      ```bash
        cargo run
      ```
3. Frontend:
    - Set the environment variables in `.env`:
      - `VITE_BACKEND_URL`: URL of the backend
      - `VITE_GH_OAUTH_CLIENT_ID`: Client ID of the GitHub OAuth App.
    - Run the frontend:
      ```bash
      npm install
      npm run dev
      ```


### Environment Variables

This project needs a [GitHub OAuth app](https://github.com/settings/developers) and a [Personal Access Token](https://github.com/settings/personal-access-tokens) of an admin of the GitHub org.

- `GH_CLIENT_ID`, `GH_CLIENT_SECRET`: Client ID and Client Secret for the GitHub OAuth application.
- `GH_ORG_NAME`: Name of the GitHub organisation
- `GH_ORG_ADMIN_TOKEN`: A GitHub PAT of an org admin
- `JWT_SECRET`: A secure string (for signing JWTs)
- `DEPLOYMENTS_DIR`: Absolute path to directory containing all the project git repos (deployed)
- `SERVER_PORT`: Port where the backend server listens to
- `CORS_ALLOWED_ORIGINS`: Frontend URLs


## Deployment
[WIP]

## Contact

<p>
📫 Metakgp -
<a href="https://slack.metakgp.org">
  <img align="center" alt="Metakgp's slack invite" width="22px" src="https://raw.githubusercontent.com/edent/SuperTinyIcons/master/images/svg/slack.svg" />
</a>
<a href="mailto:metakgp@gmail.com">
  <img align="center" alt="Metakgp's email " width="22px" src="https://raw.githubusercontent.com/edent/SuperTinyIcons/master/images/svg/gmail.svg" />
</a>
<a href="https://www.facebook.com/metakgp">
  <img align="center" alt="metakgp's Facebook" width="22px" src="https://raw.githubusercontent.com/edent/SuperTinyIcons/master/images/svg/facebook.svg" />
</a>
<a href="https://www.linkedin.com/company/metakgp-org/">
  <img align="center" alt="metakgp's LinkedIn" width="22px" src="https://raw.githubusercontent.com/edent/SuperTinyIcons/master/images/svg/linkedin.svg" />
</a>
<a href="https://twitter.com/metakgp">
  <img align="center" alt="metakgp's Twitter " width="22px" src="https://raw.githubusercontent.com/edent/SuperTinyIcons/master/images/svg/twitter.svg" />
</a>
<a href="https://www.instagram.com/metakgp_/">
  <img align="center" alt="metakgp's Instagram" width="22px" src="https://raw.githubusercontent.com/edent/SuperTinyIcons/master/images/svg/instagram.svg" />
</a>
</p>

### Maintainer(s)

The currently active maintainer(s) of this project.
See https://wiki.metakgp.org/w/Metakgp:Project_Maintainer.

- [Harsh Khandeparkar](https://github.com/harshkhandeparkar)
- [Devansh Gupta](https://github.com/Devansh-bit)

<!-- ### Past Maintainer(s)

Previous maintainer(s) of this project.
See https://wiki.metakgp.org/w/Metakgp:Project_Maintainer.

<p align="right">(<a href="#top">back to top</a>)</p> -->

## Additional documentation

  - [License](/LICENSE)
  - [Code of Conduct](/.github/CODE_OF_CONDUCT.md)
  - [Security Policy](/.github/SECURITY.md)
  - [Contribution Guidelines](/.github/CONTRIBUTING.md)

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- MARKDOWN LINKS & IMAGES -->

[contributors-shield]: https://img.shields.io/github/contributors/metakgp/maintos.svg?style=for-the-badge
[contributors-url]: https://github.com/metakgp/maintos/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/metakgp/maintos.svg?style=for-the-badge
[forks-url]: https://github.com/metakgp/maintos/network/members
[stars-shield]: https://img.shields.io/github/stars/metakgp/maintos.svg?style=for-the-badge
[stars-url]: https://github.com/metakgp/maintos/stargazers
[issues-shield]: https://img.shields.io/github/issues/metakgp/maintos.svg?style=for-the-badge
[issues-url]: https://github.com/metakgp/maintos/issues
[license-shield]: https://img.shields.io/github/license/metakgp/maintos.svg?style=for-the-badge
[license-url]: https://github.com/metakgp/maintos/blob/master/LICENSE
[wiki-shield]: https://custom-icon-badges.demolab.com/badge/metakgp_wiki-grey?logo=metakgp_logo&style=for-the-badge
[wiki-url]: https://wiki.metakgp.org
[slack-url]: https://slack.metakgp.org
