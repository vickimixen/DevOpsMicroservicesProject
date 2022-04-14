from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, status
from fastapi_pagination import Page, PaginationParams
from sqlalchemy.orm import Session

from app import schemas, crud
from app.api import deps
from app.core.security import get_current_user, CurrentUser

assignments_router = APIRouter()


@assignments_router.post("/assignments", response_model=schemas.Assignment)
def create_assignment(*,
                      db: Session = Depends(deps.get_db),
                      assignment_in: schemas.AssignmentCreate,
                      current_user: CurrentUser = Depends(get_current_user)
                      ):
    if current_user.is_student:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Students can't create assignments."
        )
    assignment = crud.create_assignment(db, assignment_in, current_user.id)
    return assignment


@assignments_router.get("/assignments",
                        response_model=Page[schemas.Assignment])
def get_assignments(*,
                    db: Session = Depends(deps.get_db),
                    params: PaginationParams = Depends(),
                    public: bool = None,
                    current_user: CurrentUser = Depends(get_current_user)
                    ):
    if current_user.is_superuser or public:
        assignments = crud.get_assignments(db, params=params, public=public)
    elif current_user.is_teacher:
        assignments = crud.get_assignments_teacher(db,
                                                   current_user.id,
                                                   params=params)
    else:
        assignments = crud.get_assignments_student(db,
                                                   current_user.id,
                                                   params=params)
    return assignments


@assignments_router.get("/assignments/{assignment_id}",
                        response_model=Optional[schemas.Assignment])
def get_assignment(*,
                   db: Session = Depends(deps.get_db),
                   assignment_id: UUID,
                   current_user: CurrentUser = Depends(get_current_user)
                   ):
    assignment = crud.get_assignment(db, assignment_id)
    if not assignment:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Assignment with id: {assignment_id} does not exist."
        )

    # Private assignments are visible to admins, owners, and attached students
    if not assignment.public and not current_user.is_superuser:
        attachment = crud.get_attachment(db, assignment_id, current_user.id)
        if str(assignment.owner_id) != current_user.id and not attachment:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail=f"You are not authorized to view this assignment."
            )

    return assignment


@assignments_router.put("/assignments/{assignment_id}",
                        response_model=schemas.Assignment)
def update_assignment(*,
                      db: Session = Depends(deps.get_db),
                      assignment_id: UUID,
                      assignment_in: schemas.AssignmentUpdate,
                      current_user: CurrentUser = Depends(get_current_user)
                      ):
    if current_user.is_student:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Only superusers or teachers can edit assignments."
        )
    assignment = crud.get_assignment(db, assignment_id)
    if not assignment:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Assignment with id: {assignment_id} does not exist."
        )
    if current_user.is_teacher and str(assignment.owner_id) != current_user.id:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Current user does not own this assignment."
        )
    assignment = crud.update_assignment(db,
                                        db_assignment=assignment,
                                        assignment_in=assignment_in)
    return assignment


@assignments_router.delete("/assignments/{assignment_id}",
                           response_model=schemas.Assignment)
def delete_assignment(*,
                      db: Session = Depends(deps.get_db),
                      assignment_id: UUID,
                      current_user: CurrentUser = Depends(get_current_user)
                      ):
    assignment = crud.get_assignment(db, assignment_id)
    if not assignment:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Assignment with id: {assignment_id} does not exist."
        )
    if current_user.is_student:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Students may not delete assignments."
        )
    if current_user.is_teacher and str(assignment.owner_id) != current_user.id:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Teachers may not delete other teachers' assignments."
        )

    attachments = crud.get_all_attached_students(db, assignment_id)
    for attachment in attachments:
        crud.delete_attachment(db, attachment.id)
    assignment = crud.delete_assignment(db, assignment_id)
    return assignment
