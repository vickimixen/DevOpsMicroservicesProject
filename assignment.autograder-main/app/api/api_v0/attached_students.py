from typing import List

from fastapi import APIRouter, Depends, HTTPException, status
from fastapi_pagination import Page, PaginationParams
from sqlalchemy.orm import Session

from app import schemas, crud
from app.api import deps
from app.core.security import get_current_user, CurrentUser
from uuid import UUID

attached_students_router = APIRouter()


@attached_students_router.post("/attachments",
                               response_model=schemas.AttachedStudent)
def create_attachment(*,
                      db: Session = Depends(deps.get_db),
                      attachment: schemas.AttachedStudentCreate,
                      current_user: CurrentUser = Depends(get_current_user)
                      ):
    assignment = crud.get_assignment(db, attachment.assignment_id)
    if not assignment:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Assignment does not exist."
        )
    if current_user.is_student and not assignment.public:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Only superusers and teachers can attach students to private assignments."
        )
    if current_user.is_teacher and str(assignment.owner_id) != current_user.id and (
        not assignment.public or str(attachment.student_id) != current_user.id
    ):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Teachers may not attach students to other teachers' private assignments."
        )
    attachment_new = crud.get_attachment(db, attachment.assignment_id, attachment.student_id)
    if attachment_new:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Student already attached to assignment."
        )
    attachment = crud.create_attachment(db,
                                        attachment.assignment_id,
                                        attachment.student_id)
    return attachment


@attached_students_router.post("/attachments/{assignment_id}",
                               response_model=List[schemas.AttachedStudent])
def create_attachments(*,
                       db: Session = Depends(deps.get_db),
                       assignment_id: UUID,
                       student_ids: List[UUID],
                       current_user: CurrentUser = Depends(get_current_user)
                       ):
    if current_user.is_student:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Only superusers and teachers can attach students to assignments."
        )

    assignment = crud.get_assignment(db, assignment_id)
    if not assignment:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Assignment does not exist."
        )

    if current_user.is_teacher and str(assignment.owner_id) != current_user.id:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Teachers may not attach students to other teachers' assignments."
        )

    attached_students = crud.get_all_attached_students(db, assignment_id)
    attached_student_ids = {att.student_id for att in attached_students}
    students_to_attach = set(student_ids) - attached_student_ids
    for student_id in students_to_attach:
        crud.create_attachment(db, assignment_id, student_id)
    return crud.get_all_attached_students(db, assignment_id)


@attached_students_router.get("/attachments/{student_id}/assignments",
                              response_model=Page[schemas.AttachedStudent])
def get_attached_assignments(*,
                             db: Session = Depends(deps.get_db),
                             student_id: UUID,
                             params: PaginationParams = Depends(),
                             current_user: CurrentUser = Depends(get_current_user)
                             ):
    if not current_user.is_superuser:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Only superusers can view attachments."
        )
    return crud.get_attached_assignments(db=db,
                                         student_id=student_id,
                                         params=params)


@attached_students_router.get("/attachments/{assignment_id}/students",
                              response_model=Page[schemas.AttachedStudent])
def get_attached_students(*,
                          db: Session = Depends(deps.get_db),
                          assignment_id: UUID,
                          params: PaginationParams = Depends(),
                          current_user: CurrentUser = Depends(get_current_user)
                          ):
    if current_user.is_student:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Only teachers and superusers can view attachments."
        )

    assignment = crud.get_assignment(db, assignment_id)
    if not assignment:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Assignment does not exist."
        )

    if current_user.is_teacher and str(assignment.owner_id) != current_user.id:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Teachers can only view students attached to their own assignments."
        )

    return crud.get_attached_students(db, assignment_id, params=params)


@attached_students_router.get("/attachments/{assignment_id}/{student_id}",
                              response_model=schemas.AttachedStudent)
def get_attachment(*,
                   db: Session = Depends(deps.get_db),
                   assignment_id: UUID,
                   student_id: UUID,
                   current_user: CurrentUser = Depends(get_current_user)
                   ):
    if current_user.is_student and str(student_id) != current_user.id:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Only teachers and superusers can view other users' attachments."
        )
    assignment = crud.get_assignment(db, assignment_id)
    if not assignment:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Assignment does not exist."
        )
    if current_user.is_teacher and str(assignment.owner_id) != current_user.id:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Teachers can only view students attached to their own assignments."
        )
    attachment = crud.get_attachment(db, assignment_id, student_id)
    if not attachment:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Attachment does not exist."
        )
    return attachment


@attached_students_router.delete("/attachments/{assignment_id}/{student_id}",
                                 response_model=schemas.AttachedStudent)
def delete_attachment(*,
                      db: Session = Depends(deps.get_db),
                      assignment_id: UUID,
                      student_id: UUID,
                      current_user: CurrentUser = Depends(get_current_user)
                      ):
    attachment = crud.get_attachment(db, assignment_id, student_id)
    if not attachment:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Attachment does not exist."
        )
    assignment = crud.get_assignment(db, assignment_id)
    if not assignment:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Assignment does not exist."
        )

    # Allow students to unsubscribe from public assignments
    unsubscribes_from_public = assignment.public and current_user.id == str(attachment.student_id)
    if current_user.is_student and not unsubscribes_from_public:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Students may not delete attachments."
        )
    if current_user.is_teacher and str(assignment.owner_id) != current_user.id:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Teachers may not remove students from other teachers' assignments."
        )
    attachment = crud.delete_attachment(db, attachment.id)
    return attachment
