import os

import requests
import dotenv

dotenv.load_dotenv()

BASE_URL = 'http://localhost'
SECRET_KEY = os.getenv('SECRET_KEY')
HEADERS = {'Authorization': f'Bearer {SECRET_KEY}'}


def which_link(tag: str) -> str:
    """
    Given a tag, queries the shortener to find the link.
    :param tag: The tag to search for.
    :return: The link.
    """
    response = requests.get(f'{BASE_URL}/which/{tag}')

    if response.status_code != 200:
        raise ValueError('Error: {}'.format(response.status_code))

    json = response.json()

    return json['url']

def get_all() -> list:
    """
    Gets all the links from the shortener.
    :return: A list of all the links.
    """
    response = requests.get(f'{BASE_URL}/all', headers=HEADERS)

    if response.status_code != 200:
        raise ValueError('Error: {}'.format(response.status_code))

    json = response.json()

    return json['routes']


def add_link(tag: str, url: str) -> bool:
    """
    Adds a link to the shortener.
    :param tag: The tag to use.
    :param url: The url to add.
    :return: True if successful, False otherwise.
    """
    response = requests.post(
        f'{BASE_URL}/route/{tag}',
        json={
            'url': url,
        },
        headers=HEADERS
    )

    if response.status_code != 200:
        return False

    return True


def update_link(tag: str, url: str) -> bool:
    """
    Updates a link in the shortener.
    :param tag: The tag to use.
    :param url: The url to add.
    :return: True if successful, False otherwise.
    """
    response = requests.put(
        f'{BASE_URL}/route/{tag}',
        json={'url': url},
        headers=HEADERS,
    )

    if response.status_code != 200:
        return False

    return True


def delete_link(tag: str) -> bool:
    """
    Deletes a link from the shortener.
    :param tag: The tag to use.
    :return: True if successful, False otherwise.
    """
    response = requests.delete(f'{BASE_URL}/route/{tag}', headers=HEADERS)

    if response.status_code != 200:
        return False

    return True
