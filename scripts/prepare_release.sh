#! /bin/bash
set -e

VERSION=$(git cliff --bumped-version)
ANNOTATION=$(git cliff --bump --unreleased)

git cliff --bump > CHANGELOG.md

git add .

git commit -m 'chore(changelog): update changelog'

git tag -a ${VERSION} -m "${ANNOTATION}"
