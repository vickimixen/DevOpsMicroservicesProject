from fastapi import APIRouter

from app import schemas
from app.core.email import Email as coreEmail
import requests

email_router = APIRouter()


@email_router.post("/email", response_model=schemas.EmailResponse)
def send_email(*, email_in: schemas.Email):
    return coreEmail.send_email(email_in)
